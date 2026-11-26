entry "ActorTypeComponent.typeId" {
  kind     = "field"
  notes    = "The entt type id of ActorTypeComponent (the FNV-1a hash of the type name `ActorTypeComponent`; one 4-byte ActorType per entity, 128 per payload page): 4F6BA419h on 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (70.26). The ActorType of an actor is not an Actor member and not a virtual: `Actor::getEntityTypeId` looks the component up in the actor's entt registry. On 26.63 it is an out-of-line function of its own (0xb9c9f0, 376 callers: `mov rdx,[rcx+10h]; mov eax,[rcx+18h]`, the pool map walk with this type id, the sparse and payload pages, `mov eax,[rax+rdx*4]`; a missing component is fatal). On 70.26 there is no such function: callers call a helper that gives a pointer to the component through an out pointer (0x1e383a0, 715 callers, `lea rdx,[rsp+..]; call; mov rax,[rsp+..]; mov eax,[rax]`), and the helper calls the storage lookup that holds the type id (0x1e76c40). On both builds rcx is the Actor, the entt registry pointer is at actor+10h and the entity id at actor+18h (Actor.mEntityContext + 8 and + 10h), so a reader finds the value with the loader's EnttLookup (storage by this type id, then the component of the entity id, 4 bytes), with the same layout mirrors as the active camera. The type info function of the component (0xc3c060 on 26.63, 0x1e607e0 on 70.26) stores this value as the entt hash and the name `ActorTypeComponent` (length 12h). Recipe 0 is the view constructor over this storage and a second one (0x362d3d0 on 26.63, 0xe370da0 on 70.26, the same bytes on both builds: `mov rsi,rdx; mov rbx,rcx; mov edx,4F6BA419h; call; mov rdi,rax; mov rcx,rbx; mov edx,8A0C2797h; call`), offset 6, the immediate of the first `mov edx`; it keeps the type id 8A0C2797h of the other storage, the only bytes that tell it from the view constructors over other pairs, and it crosses one call (wildcarded). Recipe 1 is a system constructor that assures a list of storages in a row (0x3a11200 on 26.63, 0xa074e70 on 70.26): `mov edx,<id>; call; mov [rbp+..],rax; mov rcx,reg; mov edx,85C63FFDh; call`, offset 0, the immediate of the first `mov edx`; the storages before it are RuntimeIDComponent (FC0DBBB5h) and ActorUniqueIDComponent (18F957AFh), and the next one 85C63FFDh is kept as the literal that makes it unique. It is a pattern_any: alternative 0 (26.63) has `mov [rbp+disp8],rax; mov rcx,rbx`, alternative 1 (70.26) `mov [rbp+disp32],rax; mov rcx,rdi`; each alternative matches once on its own build and not on the other (`inspect scan`). The two recipes are in two functions and share no pattern bytes. Every other displacement, immediate and branch byte is a wildcard. The check is a range over every nonzero 32-bit value. The value is the FNV-1a hash of the type name and does not change while the name stays. Class: new entry."

  recipe {
    step "pattern" {
      bytes      = "48 89 D6 48 89 CB BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 C7 48 89 D9 BA 97 27 0C 8A E8"
      offset     = 6
      literal_ok = [
        "23: the type id 8A0C2797h of the second storage of the view: the view constructors over other component pairs have the same code",
        "24: the type id 8A0C2797h of the second storage of the view: the view constructors over other component pairs have the same code",
        "25: the type id 8A0C2797h of the second storage of the view: the view constructors over other component pairs have the same code",
        "26: the type id 8A0C2797h of the second storage of the view: the view constructors over other component pairs have the same code",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern_any" {
      patterns   = [
        "BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 45 ?? 48 89 D9 BA FD 3F C6 85 E8",
        "BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 89 F9 BA FD 3F C6 85 E8",
      ]
      offset     = 0
      literal_ok = [
        "0.18: the type id 85C63FFDh of the next storage in the list: the system constructors assure many storages with the same code",
        "0.19: the type id 85C63FFDh of the next storage in the list: the system constructors assure many storages with the same code",
        "0.20: the type id 85C63FFDh of the next storage in the list: the system constructors assure many storages with the same code",
        "0.21: the type id 85C63FFDh of the next storage in the list: the system constructors assure many storages with the same code",
        "1.21: the type id 85C63FFDh of the next storage in the list, in the 70.26 shape",
        "1.22: the type id 85C63FFDh of the next storage in the list, in the 70.26 shape",
        "1.23: the type id 85C63FFDh of the next storage in the list, in the 70.26 shape",
        "1.24: the type id 85C63FFDh of the next storage in the list, in the 70.26 shape",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 1
    max = 4294967295
  }
}
