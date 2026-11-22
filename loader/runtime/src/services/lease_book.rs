use std::sync::{Mutex, MutexGuard};

use amml_abi::{AmmlInputBlocks, AmmlLeaseMode, AmmlLeaseState};

use crate::structs::{
    LeaseBookState, LeaseEntry, LeaseNotice, LeaseRefusal, LeaseTerms, NoticeKind, ResourceId, SuspendCause,
};

/// Decides which input leases are active and which are suspended.
///
/// Shared leases live together. An exclusive lease lives alone among the
/// leases that hold input (active non-preempt leases, and those suspended only
/// by a preempt), except beside the shared leases of the mods it is paired
/// with in the coexist pairs. A new exclusive lease suspends the shared
/// holders it is not paired with. Against an exclusive holder it needs a
/// higher rank: then that holder is suspended too, else the request is
/// refused and nothing changes. A shared request beside an exclusive holder
/// it is not paired with is refused. A suspended lease is judged again when
/// the lease that suspended it ends or the coexist pairs change. One preempt
/// lease at a time suspends every active lease until it ends.
///
/// The book does not know holds or move intents. Releasing the injected input
/// of a lease that is suspended or ended is the caller's work, with
/// `InjectionQueue::release`.
pub struct LeaseBook {
    state: Mutex<LeaseBookState>,
}

impl Default for LeaseBook {
    fn default() -> Self {
        Self::new()
    }
}

impl LeaseBook {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(LeaseBookState {
                leases: Vec::new(),
                policy_owner: None,
                ranking: Vec::new(),
                coexist: Vec::new(),
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, LeaseBookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records a new lease. Gives `Suspended` notices for the other owners when
    /// the new lease preempts or outranks them.
    pub fn acquire(
        &self,
        lease: ResourceId,
        owner: usize,
        owner_id: &str,
        terms: LeaseTerms,
    ) -> Result<Vec<LeaseNotice>, LeaseRefusal> {
        let mut state = self.lock();
        let preempt_live = state.leases.iter().any(|entry| entry.terms.preempt);
        let exclusive = terms.mode == AmmlLeaseMode::EXCLUSIVE;
        let mut notices = Vec::new();

        if terms.preempt {
            if preempt_live {
                return Err(LeaseRefusal::PreemptHeld);
            }
            for entry in state.leases.iter_mut().filter(|entry| entry.state == AmmlLeaseState::ACTIVE) {
                Self::suspend(entry, SuspendCause::Preempt, &mut notices);
            }
        } else {
            // An exclusive lease cannot share the frame with an active preempt lease.
            if exclusive && preempt_live {
                return Err(LeaseRefusal::ExclusiveHeld);
            }
            let conflicts: Vec<usize> = state
                .leases
                .iter()
                .enumerate()
                .filter(|(_, entry)| Self::conflicts(&state.coexist, entry, exclusive, owner_id))
                .map(|(index, _)| index)
                .collect();
            if !conflicts.is_empty() {
                let wins = exclusive
                    && conflicts.iter().all(|&index| {
                        let holder = &state.leases[index];
                        holder.terms.mode != AmmlLeaseMode::EXCLUSIVE
                            || Self::outranks(&state.ranking, owner_id, &holder.owner_id)
                    });
                if !wins {
                    return Err(LeaseRefusal::ExclusiveHeld);
                }
                // No preempt is live here, so every conflicting lease is active.
                for index in conflicts {
                    Self::suspend(&mut state.leases[index], SuspendCause::Outranked(lease), &mut notices);
                }
            }
        }

        let (start, suspended_by) = if !terms.preempt && preempt_live {
            (AmmlLeaseState::SUSPENDED, Some(SuspendCause::Preempt))
        } else {
            (AmmlLeaseState::ACTIVE, None)
        };
        state.leases.push(LeaseEntry {
            lease,
            owner,
            owner_id: owner_id.to_string(),
            terms,
            state: start,
            suspended_by,
        });
        Ok(notices)
    }

    /// Forgets one lease. Ending a preempt lease resumes the leases it suspended;
    /// a lease that waited for the ended lease is judged again. Both give
    /// `Resumed` to their owners. The ended lease's owner gets nothing.
    pub fn end(&self, lease: ResourceId) -> Vec<LeaseNotice> {
        let mut state = self.lock();
        let Some(index) = state.leases.iter().position(|entry| entry.lease == lease) else {
            return Vec::new();
        };
        state.leases.remove(index);
        Self::settle(&mut state)
    }

    /// Forgets every lease of one owner and gives up its ranking and coexist pairs. Only the other
    /// owners get notices: the owner's own listeners are gone by then.
    pub fn end_owner(&self, owner: usize) -> Vec<LeaseNotice> {
        let mut state = self.lock();
        state.leases.retain(|entry| entry.owner != owner);
        if state.policy_owner == Some(owner) {
            state.policy_owner = None;
            state.ranking.clear();
            state.coexist.clear();
        }
        Self::settle(&mut state)
    }

    /// The state of one lease. An unknown or ended lease is `LOST`.
    pub fn state(&self, lease: ResourceId) -> AmmlLeaseState {
        self.lock()
            .leases
            .iter()
            .find(|entry| entry.lease == lease)
            .map(|entry| entry.state)
            .unwrap_or(AmmlLeaseState::LOST)
    }

    /// The owner of a lease the book still knows.
    pub fn owner_of(&self, lease: ResourceId) -> Option<usize> {
        self.lock().leases.iter().find(|entry| entry.lease == lease).map(|entry| entry.owner)
    }

    /// The active leases of every owner but `owner`: how many, and the union of their blocks.
    pub fn active_of_others(&self, owner: usize) -> (u32, AmmlInputBlocks) {
        let state = self.lock();
        let others = state.leases.iter().filter(|entry| entry.owner != owner && entry.state == AmmlLeaseState::ACTIVE);
        others.fold((0u32, AmmlInputBlocks(0)), |(count, blocks), entry| {
            (count.saturating_add(1), AmmlInputBlocks(blocks.0 | entry.terms.blocks.0))
        })
    }

    /// The union of the blocks of the active leases.
    pub fn blocks_now(&self) -> AmmlInputBlocks {
        let bits = self
            .lock()
            .leases
            .iter()
            .filter(|entry| entry.state == AmmlLeaseState::ACTIVE)
            .fold(0u32, |bits, entry| bits | entry.terms.blocks.0);
        AmmlInputBlocks(bits)
    }

    /// Whether a real key passes: it is in the passthrough of every active lease that blocks keys.
    pub fn passes(&self, key: u16) -> bool {
        self.lock()
            .leases
            .iter()
            .filter(|entry| {
                entry.state == AmmlLeaseState::ACTIVE && entry.terms.blocks.0 & AmmlInputBlocks::KEYS.0 != 0
            })
            .all(|entry| entry.terms.passthrough.contains(&key))
    }

    /// Sets the ranking of mod ids, best first. The first owner to set the
    /// ranking or the coexist pairs owns both until `end_owner`; others get
    /// `RankingOwned`. Leases already held are not judged again.
    pub fn set_ranking(&self, owner: usize, ids: Vec<String>) -> Result<(), LeaseRefusal> {
        let mut state = self.lock();
        Self::take_policy(&mut state, owner)?;
        state.ranking = ids;
        Ok(())
    }

    /// Sets the pairs of mod ids whose shared and exclusive leases run
    /// together, in either order, under the ownership rule of `set_ranking`.
    /// The held leases are judged again: a shared holder that is no longer
    /// paired with an exclusive holder is suspended, and one that now is
    /// resumes when nothing else stops it.
    pub fn set_coexist(&self, owner: usize, pairs: Vec<(String, String)>) -> Result<Vec<LeaseNotice>, LeaseRefusal> {
        let mut state = self.lock();
        Self::take_policy(&mut state, owner)?;
        state.coexist = pairs;
        Ok(Self::settle(&mut state))
    }

    fn take_policy(state: &mut LeaseBookState, owner: usize) -> Result<(), LeaseRefusal> {
        if state.policy_owner.is_some_and(|policy_owner| policy_owner != owner) {
            return Err(LeaseRefusal::RankingOwned);
        }
        state.policy_owner = Some(owner);
        Ok(())
    }

    fn suspend(entry: &mut LeaseEntry, cause: SuspendCause, notices: &mut Vec<LeaseNotice>) {
        entry.state = AmmlLeaseState::SUSPENDED;
        entry.suspended_by = Some(cause);
        notices.push(LeaseNotice { owner: entry.owner, lease: entry.lease, kind: NoticeKind::Suspended });
    }

    fn resume(entry: &mut LeaseEntry, notices: &mut Vec<LeaseNotice>) {
        entry.state = AmmlLeaseState::ACTIVE;
        entry.suspended_by = None;
        notices.push(LeaseNotice { owner: entry.owner, lease: entry.lease, kind: NoticeKind::Resumed });
    }

    // A lease holds input when it is active or suspended only by a preempt; a
    // lease that waits for a winner holds nothing. Preempt leases never
    // conflict.
    fn holds(entry: &LeaseEntry) -> bool {
        matches!(entry.suspended_by, None | Some(SuspendCause::Preempt)) && !entry.terms.preempt
    }

    // Whether `entry` holds input that a lease of `requester` (exclusive or
    // not) cannot run beside: two exclusive leases always conflict, two shared
    // never, and a shared and an exclusive one unless their mods are paired.
    fn conflicts(coexist: &[(String, String)], entry: &LeaseEntry, exclusive: bool, requester: &str) -> bool {
        if !Self::holds(entry) {
            return false;
        }
        match (exclusive, entry.terms.mode == AmmlLeaseMode::EXCLUSIVE) {
            (false, false) => false,
            (true, true) => true,
            _ => !Self::paired(coexist, requester, &entry.owner_id),
        }
    }

    fn paired(coexist: &[(String, String)], first: &str, second: &str) -> bool {
        coexist.iter().any(|(a, b)| (a == first && b == second) || (a == second && b == first))
    }

    // A ranked id beats an unranked one; between two ranked ids the earlier wins;
    // two unranked ids, or the same id, do not beat each other.
    fn outranks(ranking: &[String], challenger: &str, holder: &str) -> bool {
        let rank = |id: &str| ranking.iter().position(|ranked| ranked == id);
        match (rank(challenger), rank(holder)) {
            (Some(challenger), Some(holder)) => challenger < holder,
            (Some(_), None) => true,
            _ => false,
        }
    }

    // Brings the book back to a settled state after leases ended or the
    // coexist pairs changed. First each waiting exclusive lease whose winner is
    // gone, best ranked first, resumes when no exclusive holder is left, waits
    // for the first one that is, or waits for the live preempt. Then every
    // exclusive holder suspends the shared holders it is not paired with.
    // Then each waiting shared lease whose winner is gone or now paired with
    // it is judged the same way against the holders. Last, with no preempt
    // live, the leases a preempt suspended resume.
    fn settle(state: &mut LeaseBookState) -> Vec<LeaseNotice> {
        let mut notices = Vec::new();
        let preempt_live = state.leases.iter().any(|entry| entry.terms.preempt);
        Self::judge_waiting(state, true, preempt_live, &mut notices);
        Self::displace_shared(state, &mut notices);
        Self::judge_waiting(state, false, preempt_live, &mut notices);
        if !preempt_live {
            for entry in state.leases.iter_mut().filter(|entry| entry.suspended_by == Some(SuspendCause::Preempt)) {
                Self::resume(entry, &mut notices);
            }
        }
        notices
    }

    // Judges the waiting leases of one mode whose winner no longer stops them.
    fn judge_waiting(state: &mut LeaseBookState, exclusive: bool, preempt_live: bool, notices: &mut Vec<LeaseNotice>) {
        let mut waiting: Vec<usize> = state
            .leases
            .iter()
            .enumerate()
            .filter(|(_, entry)| (entry.terms.mode == AmmlLeaseMode::EXCLUSIVE) == exclusive)
            .filter(|(_, entry)| match entry.suspended_by {
                Some(SuspendCause::Outranked(winner)) => !state.leases.iter().any(|holder| {
                    holder.lease == winner && Self::conflicts(&state.coexist, holder, exclusive, &entry.owner_id)
                }),
                _ => false,
            })
            .map(|(index, _)| index)
            .collect();
        let rank_of =
            |entry: &LeaseEntry| state.ranking.iter().position(|id| *id == entry.owner_id).unwrap_or(usize::MAX);
        waiting.sort_by_key(|&index| rank_of(&state.leases[index]));

        for index in waiting {
            let owner_id = state.leases[index].owner_id.clone();
            // A waiting exclusive lease waits only for an exclusive holder; the shared holders give way to it.
            let blocker = state
                .leases
                .iter()
                .enumerate()
                .find(|(other, entry)| {
                    *other != index
                        && Self::conflicts(&state.coexist, entry, exclusive, &owner_id)
                        && (!exclusive || entry.terms.mode == AmmlLeaseMode::EXCLUSIVE)
                })
                .map(|(_, entry)| entry.lease);
            let entry = &mut state.leases[index];
            match blocker {
                Some(blocker) => entry.suspended_by = Some(SuspendCause::Outranked(blocker)),
                None if preempt_live => entry.suspended_by = Some(SuspendCause::Preempt),
                None => Self::resume(entry, notices),
            }
        }
    }

    // Suspends each shared holder that an exclusive holder is not paired with.
    // A holder a preempt has suspended already only changes its cause.
    fn displace_shared(state: &mut LeaseBookState, notices: &mut Vec<LeaseNotice>) {
        let winners: Vec<(ResourceId, String)> = state
            .leases
            .iter()
            .filter(|entry| Self::holds(entry) && entry.terms.mode == AmmlLeaseMode::EXCLUSIVE)
            .map(|entry| (entry.lease, entry.owner_id.clone()))
            .collect();
        for (winner, winner_id) in winners {
            for index in 0..state.leases.len() {
                let entry = &state.leases[index];
                let displaced = Self::holds(entry)
                    && entry.terms.mode != AmmlLeaseMode::EXCLUSIVE
                    && !Self::paired(&state.coexist, &winner_id, &entry.owner_id);
                if !displaced {
                    continue;
                }
                let entry = &mut state.leases[index];
                if entry.state == AmmlLeaseState::ACTIVE {
                    Self::suspend(entry, SuspendCause::Outranked(winner), notices);
                } else {
                    entry.suspended_by = Some(SuspendCause::Outranked(winner));
                }
            }
        }
    }
}
