entry "Actor.aabbShapeComponent" {
  kind     = "field"
  notes    = "Offset in Actor of the pointer to its AABBShapeComponent (in the built-in component block, next to Actor.stateVectorComponent at 218h): 220h in 26.51. LeviLamina's `Actor::getAABB` is `mBuiltInComponents->mAABBShapeComponent->mAABB`. One recipe: Actor::getAttachPos (0x19f8180) reads the box for the locations 1 and 2 (`ja; mov rax,[rcx+220h]; movss xmm0,[rax+14h]; movss xmm1,[rax+8]; subss xmm0,xmm1`). Grade: getAttachPos computes the midpoint of the box from its six floats. Every displacement and branch byte is a wildcard."

  recipe {
    step "pattern" {
      bytes  = "77 ?? 48 8B 81 ?? ?? ?? ?? F3 0F 10 40 ?? F3 0F 10 48 ?? F3 0F 5C C1"
      offset = 2
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 512
    max = 640
  }
}

entry "Actor.getDimension" {
  kind      = "function"
  msvc_name = "?getDimension@Actor@@QEBAAEAVDimension@@XZ"
  notes     = "Dimension& Actor::getDimension() const, not virtual. Actor::mDimension is a WeakRef<Dimension> (pointer at 0x1C8 and control block at 0x1D0 in 26.51; 0x1F0 and 0x1F8 in 26.60, where the Actor members from here on moved by 28h). The body raises the use count of the control block with a lock cmpxchg loop, reads the pointer, drops the reference again (two virtual calls on the control block when the count reaches zero) and returns the pointer in rax. Found through the weak-reference lock prologue: 45 places in the 26.51 exe carry it, and only two are whole functions; the other one ends in a tail jump through a slot of the dimension (Actor.getDimensionBlockSource). Recipe 0 is the whole body after the register saves (lock loop, the two unlock calls, and the `mov rax,rsi; add rsp; pop; pop; ret` tail that keeps it apart from getDimensionBlockSource), then function_start_of; the push list and the `sub rsp` frame size are not in the pattern. Recipe 1 is a call site in a function of its own: `call getDimension; mov rax,[rax+disp]; mov rcx,[rax+58h]; cmp byte ptr [rcx+100h],imm; jne; movss xmm0,[rax+34h]`, read as a branch target. Every displacement, immediate and branch byte is a wildcard; recipe 0 keeps the rsi/rdi ModRM bytes of the body. The check is function_start. What changed in 6ad79ee4-17d51891a1119914: the old recipe 1 call site (`call; movsx r,word ptr [rax+disp]; movaps; call`) no longer exists in that shape, so it matched nothing; the body only moved its member displacements. Class: would have survived with a better recipe. Evidence for the 26.60 value: (1) the body pattern drafted from the 26.51 function (without the prologue) matches once on 26.60, in this function; (2) the call-site pattern drafted from a 26.51 caller of the old function matches once on 26.60 and its call target is this function."

  recipe {
    step "pattern" {
      bytes = "48 8B 91 ?? ?? ?? ?? 8B 42 ?? 44 8D 40 ?? F0 44 0F B1 42 ?? 75 ?? 48 8B B1 ?? ?? ?? ?? 48 8B B9 ?? ?? ?? ?? 48 85 FF 74 ?? F0 FF 4F ?? 75 ?? 48 8B 07 48 8B 00 48 89 F9 FF 15 ?? ?? ?? ?? F0 FF 4F ?? 75 ?? 48 8B 07 48 8B 40 ?? 48 89 F9 FF 15 ?? ?? ?? ?? 48 89 F0 48 83 C4 ?? 5F 5E C3"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 48 8B 80 ?? ?? ?? ?? 48 8B 48 ?? 80 B9 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? F3 0F 10 40 ??"
      offset = 0
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Actor.getDimensionBlockSource" {
  kind      = "function"
  msvc_name = "?getDimensionBlockSource@Actor@@QEBAAEAVBlockSource@@XZ"
  notes     = "BlockSource& Actor::getDimensionBlockSource() const. MCFOLD: its body is shared with getDimensionBlockSourceConst, so there is one address for both and about 126 callers. The body is `getDimension().getBlockSourceFromMainChunkSource()` with getDimension inlined: the weak-reference lock loop of Actor.getDimension, then a tail jump through the getBlockSourceFromMainChunkSource slot of the dimension vtable (`mov rax,[rsi]; mov rax,[rax+disp]; mov rdx,[guard]; mov rcx,rsi; add rsp; pop; pop; jmp rdx`; slot 12 in 26.51, 13 in 26.60). Recipe 0 is the body after the register saves, up to the load of the guard pointer, then function_start_of; the slot call instead of `mov rax,rsi; ret` keeps it apart from getDimension, and the push list and frame size are not in the pattern. Recipe 1 is a call site that is specific to this getter, read as a branch target: in a function of its own, the actor's level is loaded (`mov r15,[actor+disp]`, Actor::mLevel), a pointer is chosen with `lea r12,[rip+..]; cmovne r12,rdx`, then `mov rcx,actor; call getDimensionBlockSource; mov rcx,rax; mov rax,[rax]; mov rax,[rax+disp]; mov [rbp+disp],rcx; mov rdx,r12; call [guard]` (a call through a slot of the returned BlockSource with an out pointer). Every register (ModRM) byte of recipe 1 except the `mov rcx,rax; mov rax,[rax]` pair, every displacement and branch byte is a wildcard; recipe 0 keeps the rsi/rdi ModRM bytes. The check is function_start; because the body is folded, the call-site recipe is the proof that the address is the one this getter is called at. What changed in 6ad79ee4-17d51891a1119914: nothing broke (both recipes derived cold); the recipes were hardened for review findings F2 (the call site was unique only by count; it now includes the level load and the cmovne before the call) and F4 (no prologue). Evidence for the 26.60 value: (1) the body pattern drafted from the 26.51 function matches once on 26.60, in this function; (2) the call-site pattern drafted from the 26.51 caller matches once on 26.60 and its call target is this function; also its tail slot (13) is the getBlockSourceFromMainChunkSource slot that the Dimension.vtable lineup gives."

  recipe {
    step "pattern" {
      bytes = "48 8B 91 ?? ?? ?? ?? 8B 42 ?? 44 8D 40 ?? F0 44 0F B1 42 ?? 75 ?? 48 8B B1 ?? ?? ?? ?? 48 8B B9 ?? ?? ?? ?? 48 85 FF 74 ?? F0 FF 4F ?? 75 ?? 48 8B 07 48 8B 00 48 89 F9 FF 15 ?? ?? ?? ?? F0 FF 4F ?? 75 ?? 48 8B 07 48 8B 40 ?? 48 89 F9 FF 15 ?? ?? ?? ?? 48 8B 06 48 8B 40 ?? 48 8B 15 ?? ?? ?? ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "4C 8B ?? ?? ?? ?? ?? 4C 8D ?? ?? ?? ?? ?? 4C 0F 45 ?? 48 89 ?? E8 ?? ?? ?? ?? 48 89 C1 48 8B 00 48 8B 40 ?? 48 89 ?? ?? ?? ?? ?? 4C 89 ?? FF 15"
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Actor.mLevel" {
  kind     = "field"
  notes    = "Offset of `mLevel` (ILevel*) in Actor: 0x1D8 in 26.51 (LeviLamina header sum: vtable 8, then the members in order; mDimension is 0x1C8 and its control block 0x1D0, which the Actor.getDimension body confirms). Both recipes read the displacement of `mov reg,[actor+disp]` that is followed by the call of getDimensionBlockSource on the same actor and then a virtual call on the loaded level (`mov rcx,[level]; mov r10,[rcx+698h]`) in recipe 0, and a call that passes the level in recipe 1, in two different functions. Recipe 0 keeps the level in r15 and the actor in rdi. Recipe 1 is a third function: it passes the level (`mov rdx,[rdi+disp]`) to a call, moves the actor into r9 and then calls getDimensionBlockSource on it (`mov rcx,rdi; call`). Every register, displacement, immediate and call byte is a wildcard; the callee is not matched, the shape is the anchor, and each pattern is unique in the exe."

  recipe {
    step "pattern" {
      bytes  = "4C 8B ?? ?? ?? ?? ?? 48 89 ?? E8 ?? ?? ?? ?? 49 8B 0F 4C 8B 91 ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 97 ?? ?? ?? ?? 44 8B 44 24 ?? 48 89 F1 49 89 F9 E8 ?? ?? ?? ?? 48 89 F9 E8"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 1024
  }
}

entry "Actor.isAlive" {
  kind     = "vslot"
  notes    = "Slot of bool Actor::isAlive() const in the Actor vtable: 46 in 26.51 (LeviLamina header order: Actor has no base with virtuals; hasComponent is 0, the destructor 8, isBlocking 44, isDamageBlocked 45, isAlive 46, isOnFire 47). In the 39 Actor vtables that also hold the default getYawSpeedInDegreesPerSecond (slot 16), slot 46 holds `movzx eax,byte ptr [rcx+269h]; xor al,1; ret` (not dead), slots 44 and 45 hold `xor eax,eax; ret` and slot 47 a component lookup, which lines up with the header. Both recipes are virtual calls on an actor, read as a displacement and turned into a slot. Recipe 0 is in an Actor virtual (held by 112 vtables, among them the Actor vtable at slot 103): `mov esi,edx; mov rdi,rcx; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; test al,al; je`, followed by an entity-registry lookup through `[rdi+10h]`. Recipe 1 is in a function whose `this` is an actor: after the frame set-up (xmm8, xmm7 and xmm6 saves, `mov qword ptr [rbp+..],-2`) it calls the slot on `this`, and when the actor is alive it locks Actor::mDimension (`mov rcx,[rsi+1D0h]; mov eax,[rcx+8]`). A sibling function has the same call followed by the same lock; the lea rbp and the save of xmm8 right after it (the sibling also saves xmm9) keep the pattern unique. The two recipes are in different functions and share only the call instruction shape. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot."

  recipe {
    step "pattern" {
      bytes  = "89 D6 48 89 CF 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 0F 84 ?? ?? ?? ?? 48 8B 4F ?? 8B 47 ?? 48 8B 51 ??"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D AC 24 ?? ?? ?? ?? 44 0F 29 85 ?? ?? ?? ?? 0F 29 BD ?? ?? ?? ?? 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 CE 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 0F 84 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 8B 41 ??"
      offset = 47
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 30
    max = 70
  }
}

entry "Actor.getAttachPos" {
  kind      = "function"
  msvc_name = "?getAttachPos@Actor@@QEBA?AVVec3@@W4ActorLocation@Legacy@SharedTypes@@M@Z"
  notes     = "Address of `Vec3 Actor::getAttachPos(SharedTypes::Legacy::ActorLocation, float alpha) const`: 0x19f8180 in 26.51. A non-static member function that returns a class, so the Vec3 goes through a hidden pointer: this in rcx, the result buffer in rdx, the location (an int enum) in r8d, alpha in xmm3; rax holds the buffer on return. LeviLamina's `Actor::getHeadPos` is `getAttachPos(ActorLocation::Head)` with alpha 0, and `ActorLocation::Head` is 3 (LeviLamina header deps/shared_types/legacy/actor/ActorLocation.h). Found from its callers: 12 call sites pass `xorps xmm3,xmm3; mov r8d,3` and all of them call this function. The body starts with `mov edi,r8d; mov rsi,rdx; lea eax,[rdi-1]; cmp eax,1; ja` (locations 1 and 2 are computed from the AABB component at `[rcx+220h]`, the midpoint of its box, and stored to `[rsi]`, `[rsi+4]`, `[rsi+8]`); the other locations look up components through the entity context (`mov rbx,[rcx+8]`). Recipe 0 is that head of the body (`mov edi,r8d; mov rsi,rdx; lea eax,[rdi-1]; cmp eax,1; ja; mov rax,[rcx+..]; movss xmm0,[rax+..]; movss xmm1,[rax+..]`), then function_start_of. Recipe 1 is a call site that passes the head location (`mov edi,edx; mov rcx,[rcx]; mov rsi,[rcx+..]; lea rdx,[rbp+..]; xorps xmm3,xmm3; mov r8d,3; call`), read as a branch target; the location immediate 3 is kept, because the callers that pass other locations call the same function and only the immediate tells this caller apart. The two recipes are in two functions and share no pattern bytes. Every other displacement, immediate and branch byte is a wildcard. What changed in 6ad79ee4-17d51891a1119914: the location is no longer copied to edi first (`lea eax,[r8-1]` directly), so recipe 0 is a pattern_any with one alternative per shape. Evidence on 26.60: recipe 0 (the body, alternative drafted from the 26.51 head) and recipe 1 (the Head call site) reach the same function."

  recipe {
    step "pattern_any" {
      patterns = [
        "44 89 C7 48 89 D6 8D 47 ?? 83 F8 ?? 77 ?? 48 8B 81 ?? ?? ?? ?? F3 0F 10 40 ?? F3 0F 10 48 ??",
        "48 89 D6 41 8D 40 ?? 83 F8 ?? 77 ?? 48 8B 81 ?? ?? ?? ?? F3 0F 10 40 ?? F3 0F 10 48 ??",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes      = "89 D7 48 8B 09 48 8B B1 ?? ?? ?? ?? 48 8D 55 ?? 0F 57 DB 41 B8 03 ?? ?? ?? E8"
      offset     = 25
      literal_ok = ["21: mov r8d,3 is ActorLocation::Head; the callers that pass other locations call the same function"]
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Actor.mEntityContext" {
  kind     = "field"
  notes    = "Offset of `mEntityContext` (EntityContext, 24 bytes: the EntityRegistry reference, the entt registry reference, the EntityId) in Actor: 8 in 26.51. LeviLamina header (world/actor/Actor.h): the first member after the vtable pointer, so the TypedStorage sum is 8. LeviLamina's `Actor::getHealth` is `ActorAttribute::getHealth(getEntityContext())`, so every caller passes `this + mEntityContext`. Recipe 0 is a small Actor virtual (`sub rsp,28h; cmp byte ptr [rcx+269h],0; je; xor eax,eax; add rsp,28h; ret; add rcx,8; call ActorAttribute::getHealth; test eax,eax; setg al`): alive when not dead and the health is above 0; the recipe reads the immediate of `add rcx,8` (operand 1). Recipe 1 is a player function that keeps the context in rdi and calls getHealth twice (`mov rsi,rcx; lea rdi,[rcx+8]; mov rcx,rdi; call`, then `sub eax,ebx`): it reads the displacement of the lea (operand 1). The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. ActorAttribute::getHealth reads the entt registry at context+8 and the entity id at context+10h, which places the context itself and not a member of it at this offset."

  recipe {
    step "pattern" {
      bytes  = "48 83 EC ?? 80 B9 ?? ?? ?? ?? ?? 74 ?? 31 C0 48 83 C4 ?? C3 48 83 C1 ?? E8"
      offset = 20
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 89 D3 48 89 CE 48 8D 79 ?? 48 89 F9 E8"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 64
  }
}

entry "Actor.stateVectorComponent" {
  kind     = "field"
  notes    = "Offset, from the Actor, of the StateVectorComponent pointer, as game code uses it: 0x218 in 26.51. In the LeviLamina header this is Actor::mBuiltInComponents (BuiltInActorComponents, 32 bytes, at 0x218: vtable 8 plus the members in order) plus BuiltInActorComponents::mStateVectorComponent (its first member, 0); the code only ever reads the sum `[actor+0x218]`. Both recipes read the displacement of `mov rax,[rcx+disp]` and then read the position through the pointer. Recipe 0 is the Actor function that floors the feet position (`mov rax,[rcx+disp]; mov rcx,[rcx+disp2]; movss xmm0,[rax]; movss xmm6,[rax+8]; movss xmm1,[rcx+4]`, the next pointer is the shape component). Recipe 1 is an Actor function that subtracts the position of a second actor component by component (`mov rax,[rcx+disp]; mov rdx,[rdx+disp]; movss xmm1,[rax+8]; subss xmm1,[rdx+8]; movss xmm4,[rax]; ...`). They are two different functions with unwind data and share no pattern bytes. Every displacement is a wildcard. Values: 0x218 in 26.51, 0x240 in 26.60 (the Actor members from mDimension on moved by 28h; Actor.mLevel moved the same way). Recipe 1 is a pattern_any with one alternative per build shape: 26.51 subtracts the three components with scalar `movss/subss`; 26.60 subtracts z as a scalar, then x and y as a pair (`movsd xmm4,[rax]; movsd xmm3,[rdx]; subps xmm4,xmm3`). What changed in 6ad79ee4-17d51891a1119914: the compiler vectorised the x/y part of recipe 1's function, so its old pattern matched nothing; recipe 0 still matches. Class: real code change (code shape); the layout move of 28h is a real layout change. Evidence for 0x240 on 26.60: (1) recipe 0, drafted from its 26.51 function, matches once on 26.60 and reads [actor+240h]; (2) the first four instructions of recipe 1's 26.51 function (the two state vector loads of two actors and the z subtraction), drafted from 26.51, match once on 26.60 and read [rcx+240h] and [rdx+240h]; the call site of Actor.getDimensionBlockSource loads mLevel at 200h (1D8h in 26.51), the same 28h shift."

  recipe {
    step "pattern" {
      bytes  = "48 8B 81 ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? F3 0F 10 00 F3 0F 10 70 ?? F3 0F 10 49 ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 81 ?? ?? ?? ?? 48 8B 92 ?? ?? ?? ?? F3 0F 10 48 ?? F3 0F 5C 4A ?? F3 0F 10 20 F3 0F 10 68 ?? F3 0F 5C 6A ?? F3 0F 5C 22",
        "48 8B 81 ?? ?? ?? ?? 48 8B 92 ?? ?? ?? ?? F3 0F 10 48 ?? F3 0F 5C 4A ?? 0F 28 D1 F3 0F 59 D1 F2 0F 10 20 F2 0F 10 1A 0F 5C E3",
      ]
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 2048
  }
}

entry "Actor.actorRotationComponent" {
  kind     = "field"
  notes    = "Offset, from the Actor, of the ActorRotationComponent pointer, as game code uses it: 0x228 in 26.51. LeviLamina header sum: Actor::mBuiltInComponents at 0x218 plus BuiltInActorComponents::mActorRotationComponent at 0x10 (after mStateVectorComponent and mAABBShapeComponent, 8 bytes each) = 0x228; the code reads only the sum `[actor+0x228]`. Recipe 0 is a player function that throws an item: it reads Actor::mDimension (0x1C8/0x1D0) of the same object, then `mov rax,[rcx+disp]; movss xmm6,[rax]; movss xmm7,[rax+4]; lea rcx,[rbp+..]; movss xmm2,[rip+..]; mov rdx,rdi; call`, and builds the throw velocity from sin/cos of the two angles. Recipe 1 is Actor::getYawSpeedInDegreesPerSecond (virtual; 41 Actor vtables hold it): from `mov rax,[rcx+disp]; movss xmm0,[rax+4]; subss xmm0,[rax+0Ch]; addss xmm0,[180.0]; movss xmm6,[360.0]; movaps xmm1,xmm6; call [fmodf]` (wrapDegrees(mRot.y - mRotPrev.y) * 20). Recipe 2 is a function that sets the rotation and the previous rotation to the same values (`mov rax,[rcx+disp]; movss [rax],xmm7; movss [rax+4],xmm0; mov rax,[rcx+disp]; movss [rax+8],xmm7; movss [rax+0Ch],xmm0; lea rax,[rip+..]; mov [rbp],rax; mov qword ptr [rbp+8],0Ch`); a second setter with the same first six instructions exists (xmm6, then `mov dword ptr [rcx+3B4h],1Ch`), so the trailing string setup keeps this one unique. The three are different functions and share no anchor. Every displacement, immediate and branch byte is a wildcard. Values: 0x228 in 26.51, 0x250 in 26.60 (+28h, the same shift as Actor.mLevel and Actor.stateVectorComponent). Recipe 0 is a pattern_any: 26.51 loads pitch and yaw with two `movss` (xmm6, xmm7), 26.60 loads both with one `movsd xmm6,[rax]`; the load of the pointer has its ModRM byte wildcarded (26.51 `[rcx+disp]`, 26.60 `[rdi+disp]`). Recipe 1 no longer holds the `sub rsp; movaps` prologue (review M4). What changed in 6ad79ee4-17d51891a1119914: the throw function was vectorised and its register use changed, so recipe 0 matched nothing; recipes 1 and 2 still match. Class: real code change (code shape); the 28h move is a real layout change. Evidence for 0x250 on 26.60: (1) recipe 1 (Actor::getYawSpeedInDegreesPerSecond), drafted from 26.51, matches once on 26.60 and reads [actor+250h]; (2) recipe 2 (the rotation setter), drafted from 26.51, matches once on 26.60 and reads [actor+250h]; and in the 26.60 throw function (the same `ret` tail, `lea rcx,[rbp-50h]; movss xmm2,[rip+..]; mov rdx,rdi; call` as in 26.51) the rotation pointer is read at +250h."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B ?? ?? ?? ?? ?? F3 0F 10 30 F3 0F 10 78 ?? 48 8D 4D ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 FA E8",
        "48 8B ?? ?? ?? ?? ?? F2 0F 10 30 48 8D 4D ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 FA E8",
      ]
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 81 ?? ?? ?? ?? F3 0F 10 40 ?? F3 0F 5C 40 ?? F3 0F 58 05 ?? ?? ?? ?? F3 0F 10 35 ?? ?? ?? ?? 0F 28 CE FF 15"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 81 ?? ?? ?? ?? F3 0F 11 38 F3 0F 11 40 ?? 48 8B 81 ?? ?? ?? ?? F3 0F 11 78 ?? F3 0F 11 40 ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 2048
  }
}

entry "Actor.getViewVector" {
  kind     = "function"
  notes    = "Vec3 Actor::getViewVector(float a) const: rcx the actor, rdx the result buffer (the Vec3 comes back through the hidden pointer), xmm2 the interpolation factor. The body looks up the actor's rotation component in the entity registry (type hash 75DF36B7h; 16-byte rows: rotation x and y, then the previous x and y), interpolates each angle with a wrap to -180..180 (`fmodf`), and builds the direction from the 65536-entry sine table (x = -sin(yaw) cos(pitch) form, y from the pitch), then writes x, y and z to the buffer and returns it. A missing component is fatal. Recipe 0 is the head of the body: `movaps xmm6,xmm2; mov rsi,rdx; mov rdx,[rcx+..]; mov eax,[rcx+..]; mov rcx,[rdx+..]; mov r8,[rdx+..]; sub r8,rcx; shr r8,..; dec r8d; and r8d,..` (the factor and the buffer saved, then the first steps of the registry lookup); the hash is a wildcard. Recipe 1 is the client pick function, the same function that reads Level::getHitResult: `lea rdx,[rbp+..]; movss xmm2,[rip+..]; mov rcx,rdi; call` (the factor 1.0), read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start."

  recipe {
    step "pattern" { bytes = "0F 28 F2 48 89 D6 48 8B 51 ?? 8B 41 ?? 48 8B 4A ?? 4C 8B 42 ?? 49 29 C8 49 C1 E8 ?? 41 FF C8 41 81 E0" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 95 ?? ?? ?? ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 F9 E8"
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Actor.getRuntimeID" {
  kind     = "function"
  notes    = "ActorRuntimeID Actor::getRuntimeID() const: rcx the actor, rdx the 8-byte result buffer (a member function returns the class through the hidden pointer; the body writes [rdx] and returns rdx). It looks up the runtime id component (type hash 0FC0DBBB5h) and copies its 8 bytes; a missing component is fatal. Its twin with the same lookup is the setter (it stores rdx into the component). Recipe 0 is the ClientInstance function that opens the inventory: the InteractPacket target id comes from this call on the player or on the ridden actor (`lea rdx,[rbp-..]; mov rcx,r14; jmp; lea rdx,[rbp-..]; mov rcx,rbx; call`), read as the call target. Recipe 1 is a packet constructor that fills a runtime id member in place (`mov [rdi+..],rax; lea rdx,[rdi+..]; mov rcx,rsi; call`), read as the call target. The two recipes are in two functions. Every displacement and branch byte is a wildcard. The check is function_start. Recipe 0 is now a pattern_any, one alternative per shape of the open-inventory function, the call at offset 12 in both: alternative 0 (26.51) the shared call after `mov rcx,r14; jmp`; alternative 1 (26.60) the ridden-actor call after a `jmp` (the function calls it twice, once per branch), with the action and mode load after it (`mov rax,imm64`). What changed in 6ad79ee4-17d51891a1119914 (26.60): the two branches no longer share one call; recipe 1 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "4C 89 F1 EB ?? 48 8D 55 ?? 48 89 D9 E8",
        "E9 ?? ?? ?? ?? 48 8D 55 ?? 48 89 D9 E8 ?? ?? ?? ?? 48 B8",
      ]
      offset = 12
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 47 ?? 48 8D 57 ?? 48 89 F1 E8"
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Actor.setYHeadRotations" {
  kind      = "function"
  msvc_name = "?setYHeadRotations@Actor@@QEAAXMM@Z"
  notes     = "Address of `void Actor::setYHeadRotations(float yHeadRot, float oldYHeadRot)`: 0x1a29fa0 in 26.51 (this in rcx, the two angles in xmm1 and xmm2, no return value). Not virtual. The head rotation is in an ECS component (ActorHeadRotationComponent, type hash 0xBABE7211, 8-byte rows), not in a built-in Actor pointer, so the mod calls this function. It is a leaf without .pdata on both builds. The body takes the entity context (Actor.mEntityContext, +8: `mov rdx,[rcx+10h]; mov eax,[rcx+18h]`), looks up the component pool by its hash, finds the entity's row through the sparse set and stores xmm1 to row+0 and xmm2 to row+4, then `ret`. Its neighbour before it (0x1a29ed0) is the same lookup with only the xmm1 store: that is setYHeadRot(float). Recipe 0 is the whole body from its first byte, offset 0: the double store at its end (`movss [rax+rdx*8],xmm1; movss [rax+rdx*8+..],xmm2; ret`) tells it apart from setYHeadRot and the other component setters; the type hash, the registry displacements, the masks and the shifts are wildcards, and the body still matches once on each build. Recipe 1 is a caller that sets the rotation to (0, yaw) and then the head rotation to the yaw (`mov dword ptr [rax],..; mov rax,[actor+..]; movss [rax+..],xmm1; mov rcx,actor; movaps xmm2,xmm1; call`), read as a branch target at index 25. It is a pattern_any, one alternative per shape: on 26.51 the actor is in rdi (the alternative starts one byte earlier with a wildcard), on 26.60 in r12. Each alternative matches once on its own build and not on the other. The two recipes are in two functions (the body against its caller) and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is leaf_start: 16-byte aligned, after int3 padding or a .pdata range that ends in a ret, inside no other .pdata range, and the target of at least one direct call (4 callers on 26.51, 6 on 26.60). Values: 0x1a29fa0 in 26.51, 0xbd3c20 in 26.60. What changed in 6ad79ee4-17d51891a1119914: the body is the same; only its address and the caller's registers changed. Evidence on 26.60: (1) the 26.51 body, drafted with its build-specific bytes as wildcards, matches once on 26.60, at a leaf start; (2) the 26.51 caller (0x4bfeec) sets the rotation and then calls this function with both angles in xmm1 and xmm2; on 26.60 the same sequence (0x4e0ca22) calls the leaf found by (1)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 51 ?? 8B 41 ?? 48 8B 4A ?? 4C 8B 42 ?? 49 29 C8 49 C1 E8 ?? 41 FF C8 41 81 E0 ?? ?? ?? ?? 4E 8D 0C C1 48 8B 4A ?? 0F 1F 84 00 ?? ?? ?? ?? 4D 8B 01 49 83 F8 ?? 0F 84 ?? ?? ?? ?? 49 C1 E0 ?? 4E 8D 0C 01 42 81 7C 01 ?? ?? ?? ?? ?? 75 ?? 4C 01 C1 48 39 4A ?? 74 ?? 48 8B 49 ?? 48 85 C9 74 ?? 89 C2 81 E2 ?? ?? ?? ?? 41 89 D0 41 C1 E8 ?? 4C 8B 49 ?? 4C 8B 51 ?? 4D 29 CA 49 C1 FA ?? 4D 39 D0 73 ?? 4F 8B 04 C1 4D 85 C0 74 ?? 81 E2 ?? ?? ?? ?? 25 ?? ?? ?? ?? 41 8B 14 90 31 D0 3D ?? ?? ?? ?? 77 ?? 48 8B 41 ?? 89 D1 C1 E9 ?? 81 E1 ?? ?? ?? ?? 48 8B 04 08 48 85 C0 74 ?? 81 E2 ?? ?? ?? ?? 83 E2 ?? F3 0F 11 0C D0 F3 0F 11 54 D0 ?? C3"
      offset = 0
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "?? C7 00 ?? ?? ?? ?? 48 8B 87 ?? ?? ?? ?? F3 0F 11 48 ?? 48 89 F9 0F 28 D1 E8",
        "C7 00 ?? ?? ?? ?? 49 8B 84 24 ?? ?? ?? ?? F3 0F 11 48 ?? 4C 89 E1 0F 28 D1 E8",
      ]
      offset = 25
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "leaf_start" {}
}

entry "Actor.getItemInHandSlot" {
  kind      = "function"
  msvc_name = "?getItemInHandSlot@Actor@@QEBAAEBVItemStack@@W4HandSlot@@@Z"
  notes     = "ItemStack const& Actor::getItemInHandSlot(HandSlot slot) const, not virtual: rcx the Actor, dl the HandSlot (0 Mainhand, 1 Offhand), the ItemStack reference in rax. LeviLamina header world/actor/Actor.h. The body: for slot 1 it takes the hand SimpleContainer (ActorEquipmentComponent::mHand) from the entity context (`add rcx,8; call <EntityContext ActorEquipmentComponent hand getter>`, the EnTT lookup with the type hash 0B06141A9h, FNV-1a of `ActorEquipmentComponent`) and tail-jumps to its getItem (Container slot 7, `[rcx+38h]`) with edx = 1; for any other slot it tail-jumps to the virtual Actor::getCarriedItem (`[rax+268h]`, slot 77). Found from the EnTT hash: the game inlines `hand->getItem(1)` in about 170 places, and the only small .pdata functions with the shape are this one, Actor::getCarriedItem (the same tail with edx = 0, in 114 vtables) and a wrapper that first loads an Actor from [rcx+8] (0x3f9cac0 on 26.63, in one vtable). Recipe 0 is the body from its first byte (`sub rsp,imm; cmp dl,1; je; movzx eax,dl; test eax,eax; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,[rip+guard]; add rsp,imm; jmp rdx; add rcx,imm; call`), offset 0, ended at the call; the `sub rsp` right before `cmp dl,1` keeps the wrapper (which has `mov rcx,[rcx+8]` there) out, and the kept `cmp dl,1` immediate keeps getCarriedItem out. Recipe 1 is a call site in a function of its own (an item use handler that stores a bool at +42h and then reads the stack in the chosen hand): `mov [reg+disp8],al; mov rcx,[reg+disp8]; movzx edx,byte ptr [rbp+disp32]; call`, read as the branch target; the ModRM bytes are wildcards because 26.51 uses rcx there and 26.60 and later use rbx. The two recipes are in two functions and share no pattern bytes. Every other displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 0 does not end with function_start_of, so it counts). The mod reads the offhand stack as getItemInHandSlot(player, 1) on the game thread; the reference is the container's own slot, so it changes when the slot is set again. Values: 0x1a0d000 on 6aa482fd-2658118eba0dbac5 (26.51), 0xbb6ba0 on 6ad79ee4-17d51891a1119914 (26.60), 0xbb6bb0 on 6aea437e-7eff13daabb430da (26.62), 0xbb6ae0 on 6aed1906-b127418e74d2f2f9 (26.63). The body is the same on all four builds. Evidence on 26.63: (1) the body matches the 26.51 function byte for byte outside the call and guard displacements; (2) the call site, found from the 26.51 caller (0x259d580, the same `mov [reg+42h],al` store and HandSlot byte load), calls it on every build. Minecraft Preview 1.26.70.23 (6ae4eeee-4dd4fa7acacb4c41): 0x1c87870. There the function has a stack cookie and an assert path: `push rsi; sub rsp,imm; mov rax,[rip+cookie]; xor rax,rsp; mov [rsp+..],rax; cmp dl,1; je; movzx eax,dl; test eax,eax; jne <assert, then getCarriedItem>; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,[rip+guard]`. For slot 1 the EnTT lookup is no longer inline: it does `add rcx,8`, passes the context and the same hash 0B06141A9h to a generic component lookup (0x60a49c0), then `mov rcx,[rax]` (mHand) and tail-jumps to `[rax+38h]` with edx = 1 after the cookie check. Recipe 0 is a pattern_any with one alternative per shape, both from the function's first byte at offset 0: alternative 0 the 26.51..26.63 body above, alternative 1 the Preview prologue and body up to the guard load. In alternative 1 the near `jne` to the assert (`0F 85`) keeps out the Preview setter next to it (0x1c87780: a short `jne`, then `[rax+260h]`), and `mov rax,[rcx]` (ModRM 01) keeps out the Preview wrapper that first loads an Actor into rsi (0x58001e0, `mov rax,[rsi]`). Recipe 1 is a pattern_any too, both alternatives a direct call at offset 14 read as the branch target: alternative 0 the item use handler call above (26.51..26.63; on the Preview that handler, 0x23eeab0, holds the HandSlot in r13d, `mov [rbx+42h],al; mov rcx,[rbx+8]; mov edx,r13d; call`, so alternative 0 matches nothing there); alternative 1 a Preview caller of its own (0xf473f80: `mov [rbp+disp32],r15d; movzx edx,byte ptr [rsi+disp8]; mov rcx,rbx; call`, the ModRM bytes of the movzx and the mov wildcarded). Each alternative matches once on its own builds and nothing on the others. Evidence for 0x1c87870 on the Preview: (1) its body, read against the 26.63 function 0xbb6ae0, has the same two tails: vtable +268h (Actor::getCarriedItem) for slot 0, and for slot 1 the ActorEquipmentComponent hand container (hash 0B06141A9h) with getItem at +38h and edx = 1; (2) the 26.63 caller of recipe 1 (the item use handler 0x376c780, whose two calls both go to 0xbb6ae0) is 0x23eeab0 on the Preview, with the same `mov [rbx+42h],al; mov rcx,[rbx+8]` before the call, and both of its calls go to 0x1c87870. What changed in 6ae4eeee-4dd4fa7acacb4c41: the stack cookie, the assert and the out-of-line component lookup changed the body, and the caller of recipe 1 moved the HandSlot to a register. Class: real code change (code shape); the meaning of the function is the same."

  recipe {
    step "pattern_any" {
      patterns   = [
        "48 83 EC ?? 80 FA 01 74 ?? 0F B6 C2 85 C0 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15 ?? ?? ?? ?? 48 83 C4 ?? 48 FF E2 48 83 C1 ?? E8",
        "56 48 83 EC ?? 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 80 FA 01 74 ?? 0F B6 C2 85 C0 0F 85 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15",
      ]
      offset     = 0
      literal_ok = [
        "0.6: cmp dl,1: the HandSlot::Offhand test; Actor::getCarriedItem has the same container tail without a compare",
        "1.22: cmp dl,1: the HandSlot::Offhand test in the Preview shape; the Preview getCarriedItem has no compare",
      ]
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "88 ?? ?? 48 8B ?? ?? 0F B6 95 ?? ?? ?? ?? E8",
        "44 89 BD ?? ?? ?? ?? 0F B6 ?? ?? 48 89 ?? E8",
      ]
      offset   = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
