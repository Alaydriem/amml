entry "LocalWorldStarter.startLocalWorld" {
  kind  = "function"
  notes = "World::StartLocalWorldResult World::LocalWorldStarter::startLocalWorld(const World::WorldID &): this in rcx (the LocalWorldStarter that WorldSystem.mLocalWorldStarter points to), the WorldID in rdx (a World::WorldID, NewType<std::string>: one MSVC std::string holding the level id, the world folder name), the result in eax (StartLocalWorldResult: 0 Ok, 1 WorldNotInLevelList, 2 StartupAlreadyInProgress, 3 GameServerStarterNotReady, 9 WorldSavedInNewerVersion, 12 HasNotConfirmedPlatformLockedContent, 13 WorldNotFoundInFileSystem, 14 WorldHasSlimCacheData). It is the call the Ore UI Play button reaches: the WorldStartupFacet facet function \"startLocalWorld\" (registered in 0x7ac6ac0 on 26.63) queues a task on its \"Start Local World Group\" TaskGroup (a \"Main Thread\" worker pool), and the task body (0x7ae8f80, slot 2 of the callable at 0xf335f40) calls this function with the facet's mLocalWorldStarter ([facet+18h]), then pushes the \"start_local_world_progress\" loading screen (WorldStartupFacet::_pushLoadingScreen, 0x7acb490). The other direct caller is the backup-then-start continuation of LocalWorldStarter::backupThenStartLocalWorld (0x2545810). The body matches the LeviLamina layout of World::LocalWorldStarter and the result enum: it returns mStartLocalWorldErrorOverride when set ([this+B4h] has_value, [this+B0h] value), 2 while mServerStartupResult ([this+A0h]) is not done, 3 when mGameServerStartup ([this]) slot 1 says not ready, 1 when ILevelListCache ([this+8h]) slot 18 gives no LevelSummary for the id, 9 or 13 after the version and Core::FileSystem ([this+58h]) checks, then calls IGameServerStartup::startLocalServerAsyncFromSummary (0xa04300 on 26.63, its only caller) and stores the Async in mServerStartupResult, and returns 0. Recipe 0 is a body anchor after the not-ready test: `mov ecx,eax; mov eax,imm (3); test cl,cl; je rel32; mov rcx,[rsi+8]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,rdi; call [guard]` (the first ILevelListCache call), then `function_start_of`; every immediate, displacement, branch and register ModRM byte that can change is a wildcard, and it ends at the call. Recipe 1 shares no bytes with it: the one direct call in the backupThenStartLocalWorld continuation, `mov rax,[rcx+disp8]; lea rdx,[rcx+disp8]; mov rsi,rcx; mov rcx,rax; call`, offset 14, the branch target of the call; the register moves are wildcards. Both recipes are unique on 26.63 and agree. The check is `function_start` (recipe 1 does not end in `function_start_of`, so it counts). Values: 0x251abc0 on 6aed1906-b127418e74d2f2f9 (26.63) and 0x251ae60 on 6aea437e-7eff13daabb430da (26.62); both recipes agree on both builds. Found on 26.63; 26.60 (6ad79ee4-17d51891a1119914) was not checked, because no 26.60 exe was available."

  recipe {
    step "pattern" {
      bytes = "89 ?? B8 ?? ?? ?? ?? 84 ?? 0F 84 ?? ?? ?? ?? 48 8B ?? ?? 48 8B 01 48 8B 40 ?? 48 89 ?? FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 41 ?? 48 8D 51 ?? 48 89 ?? 48 89 ?? E8"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
