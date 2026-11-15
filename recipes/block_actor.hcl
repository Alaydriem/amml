entry "BlockActor.loadStatic" {
  kind  = "function"
  notes = "static std::shared_ptr<BlockActor> BlockActor::loadStatic(BlockType const&, BlockPos const&, ILevel&, CompoundTag const&, DataLoadHelper&): rcx = the hidden return pointer (16 bytes), rdx = the BlockType, r8 = the position, r9 = the ILevel, [rsp+20h] at the call = the CompoundTag, [rsp+28h] = the DataLoadHelper; it returns rcx in rax. The body calls the global block actor factory std::function (impl pointer at 0x1296cf98 on 26.63, slot 2 `[rax+10h]` = _Do_call, with rdx = a local out pointer, r8 = the position, r9 = the BlockType); that function is VanillaBlockActorFactory.createBlockActor. It returns an empty pointer when the global is empty or the factory gives none. Then it reads the `id` string of the tag (`lea rax,[rip+..]; mov qword ptr [rbp-48h],2`), hashes it (FNV-1a) and looks it up. How it was found: MovingBlockActor::load (0x4c426d0, the only function that loads the string `movingEntity`) reads the `movingEntity` compound and its x, y and z, calls DataLoadHelper slot 2 (loadBlockPosition), and then calls this function with the moving block's BlockType (`mov rax,[rsi+0F8h]; mov rdx,[rax+68h]`), the position, the level, the tag and the helper, and stores the result in its block actor member (+108h). Callers on 26.63: 0x1fa4940, 0x385c690, 0x4c426d0, 0x69869d0, 0x781b820, 0x88fb880. Recipe 0 is the head (`mov rsi,rcx; mov rcx,[rip+..]; test rcx,rcx; je rel32; mov rdi,r9; mov r14,rdx; mov rax,[rcx]; mov rax,[rax+disp8]; mov rdx,rbp; mov r9,r14; call [guard]`), then function_start_of. Recipe 1 is the call in MovingBlockActor::load (`mov rax,[rsi+disp32]; mov rdx,[rax+disp8]; mov [rsp+28h],rbx; mov [rsp+20h],r14; lea rcx,[rsp+..]; mov r8,r15; mov r9,rdi; call`), read as a branch target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Value 0x3301fb0 on 26.63 (6aed1906-b127418e74d2f2f9)."

  recipe {
    step "pattern" {
      bytes = "48 89 CE 48 8B 0D ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 4C 89 CF 49 89 D6 48 8B 01 48 8B 40 ?? 48 89 EA 4D 89 F1 FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 86 ?? ?? ?? ?? 48 8B 50 ?? 48 89 5C 24 ?? 4C 89 74 24 ?? 48 8D 4C 24 ?? 4D 89 F8 49 89 F9 E8"
      offset = 32
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BlockActor._getRenderComponent" {
  kind  = "vslot"
  notes = "Slot of IVanillaRenderBlockActorComponent* BlockActor::_getRenderComponent() (the non-const overload) in the BlockActor vtable: 21 on 26.63 (6aed1906-b127418e74d2f2f9). rcx = the block actor; it returns the component pointer (an interior base of the actor) in rax, or null. The LeviLamina header order gives 21 for the first of the two overloads after _playerCanUpdate (dtor 0, load 1, ... _playerCanUpdate 20). Both call sites pass the result as the component (r9) of BlockActorRenderDispatcher.render, which then calls component slots 2, 4 and 6, so the slot belongs to BlockActor and returns the render component. Recipe 0 is in the block selection overlay draw (0x539b520): the block actor in rax (`mov rdx,rax; mov rax,[rax]; mov rax,[rax+disp32]; mov rcx,rdx; mov [rbp+..],rdx; call [guard]`), offset 6; the result goes to the local that is later passed to the first render form, and the same function tests the actor type at +14h (BlockActor::mType). Recipe 1 is in MovingBlockActorRenderer's draw (0x6a83a10), on the moving block's actor member (`mov qword ptr [rax+..],imm32; mov rcx,[r12+disp32]; test rcx,rcx; je rel32; mov rax,[rcx]; mov rax,[rax+disp32]; call [guard]`), offset 28; the result is passed straight to the renderPos form. Both displacements are A8h and are turned into a slot. The two recipes are in two functions. Every displacement and immediate is a wildcard. The range check 17..26 covers the BlockActor virtuals from triggerEvent to the last component getter."

  recipe {
    step "pattern" {
      bytes  = "48 89 C2 48 8B 00 48 8B 80 ?? ?? ?? ?? 48 89 D1 48 89 95 ?? ?? ?? ?? FF 15"
      offset = 6
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 C7 40 ?? ?? ?? ?? ?? 49 8B 8C 24 ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
      offset = 28
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 17
    max = 26
  }
}
