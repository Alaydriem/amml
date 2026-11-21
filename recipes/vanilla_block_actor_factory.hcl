entry "VanillaBlockActorFactory.createBlockActor" {
  kind  = "function"
  notes = "std::shared_ptr<BlockActor> VanillaBlockActorFactory::createBlockActor(BlockPos const&, BlockType const&), a namespace function: rcx = the hidden return pointer (16 bytes: element, control block), rdx = the position, r8 = the BlockType; it returns rcx in rax. The body calls BlockType::getBlockEntityType (0x35ae6c0 on 26.63) on r8, subtracts 1 and switches over 62 BlockActorType values (`cmp eax,3Dh; ja; jump table`); each case calls a make_shared helper with (&local, pos) and moves the result to [rcx]. Type 0 or an unknown type gives an empty pointer. How it was found: BlockActor::loadStatic (0x3301fb0) calls a global std::function<shared_ptr<BlockActor>(BlockPos const&, BlockType const&)> (impl pointer at 0x1296cf98); the function that fills that global (0x7012270) builds a std::function from a function pointer (`lea rax,[impl vtable]; mov [rbp-8],rax; lea rcx,[rip+..]; mov [rbp],rcx`), and the pointer is this function. The exe also holds the RTTI name of that function pointer type (`.P6A?AV?$shared_ptr@VBlockActor@@@std@@AEBVBlockPos@@AEBVBlockType@@@Z`). The function has no direct callers on 26.63. Recipe 0 is the switch head after the call of getBlockEntityType (`movzx eax,al; dec eax; cmp eax,imm8; ja rel32; lea rcx,[rip+..]; movsxd rax,[rcx+rax*4]; add rax,rcx; jmp rax`, then the first case up to its call), then function_start_of. Recipe 1 is the std::function build in 0x7012270, offset 11, the RIP target of `lea rcx,[rip+..]`. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Value 0x7150f90 on 26.63 (6aed1906-b127418e74d2f2f9). What changed in 6afa4e47-dadc375797a3d31a: the function gained an argument and its body changed. It is now `createBlockActor(out, BlockPos const&, bool, BlockType const&)` (the position in rdx, a bool in r8b, the BlockType in r9: the BlockType moved from r8 to r9, a real code change that the loader and mod code must know about before they call it). It no longer calls BlockType::getBlockEntityType: it reads the block entity type byte straight from the BlockType (`movzx eax,byte ptr [rbx+disp]`), and first tries two other factories on the BlockType (two calls on `[BlockType+30h]`, then a helper that makes the actor); the 62-way switch (`dec eax; cmp eax,3Dh; ja; jump table`) and its first case are unchanged. Recipe 0 has a second alternative for the switch head with that load. In recipe 1 the std::function that holds the factory is built differently (the vtable pointer is stored at [rbp-10h] and the function pointer at [rbp-8]; `lea rax,[rip+..]; mov [rbp-..],rax; lea rax,[rip+..]; mov [rbp-..],rax; mov [rbp+..],r15; mov rcx,r15; call; nop; mov rax,[r14]; test rax,rax; je`), so alternative 1 reads the RIP target of the second `lea rax` in the function that fills the global std::function. Class: real code change (argument added); the recipes would have survived with a better recipe for the shape. Evidence on 6afa4e47-dadc375797a3d31a: (1) the function that holds the second alternative of recipe 1 is the only function on the new build that loads the address of the function that recipe 0 finds (one RIP-relative reference on each build, from the function that fills the std::function); (2) the 62-way switch with the same first case (`lea rbx,[rbp-20h]; mov rcx,rbx; mov rdx,rdi; call`) is in the same function that recipe 1 names."

  recipe {
    step "pattern_any" {
      patterns = [
        "0F B6 C0 FF C8 83 F8 ?? 0F 87 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 63 04 81 48 01 C8 FF E0 48 8D 5D ?? 48 89 D9 48 89 FA E8",
        "0F B6 83 ?? ?? ?? ?? FF C8 83 F8 ?? 0F 87 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 63 04 81 48 01 C8 FF E0 48 8D 5D ?? 48 89 D9 48 89 FA E8",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 4C 89 7D ?? 48 8B 0D",
        "48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 4C 89 7D ?? 4C 89 F9 E8 ?? ?? ?? ?? 90 49 8B 06 48 85 C0 0F 84",
      ]
      offset = 11
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "function_start" {}
}
