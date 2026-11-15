entry "VanillaBlockActorFactory.createBlockActor" {
  kind  = "function"
  notes = "std::shared_ptr<BlockActor> VanillaBlockActorFactory::createBlockActor(BlockPos const&, BlockType const&), a namespace function: rcx = the hidden return pointer (16 bytes: element, control block), rdx = the position, r8 = the BlockType; it returns rcx in rax. The body calls BlockType::getBlockEntityType (0x35ae6c0 on 26.63) on r8, subtracts 1 and switches over 62 BlockActorType values (`cmp eax,3Dh; ja; jump table`); each case calls a make_shared helper with (&local, pos) and moves the result to [rcx]. Type 0 or an unknown type gives an empty pointer. How it was found: BlockActor::loadStatic (0x3301fb0) calls a global std::function<shared_ptr<BlockActor>(BlockPos const&, BlockType const&)> (impl pointer at 0x1296cf98); the function that fills that global (0x7012270) builds a std::function from a function pointer (`lea rax,[impl vtable]; mov [rbp-8],rax; lea rcx,[rip+..]; mov [rbp],rcx`), and the pointer is this function. The exe also holds the RTTI name of that function pointer type (`.P6A?AV?$shared_ptr@VBlockActor@@@std@@AEBVBlockPos@@AEBVBlockType@@@Z`). The function has no direct callers on 26.63. Recipe 0 is the switch head after the call of getBlockEntityType (`movzx eax,al; dec eax; cmp eax,imm8; ja rel32; lea rcx,[rip+..]; movsxd rax,[rcx+rax*4]; add rax,rcx; jmp rax`, then the first case up to its call), then function_start_of. Recipe 1 is the std::function build in 0x7012270, offset 11, the RIP target of `lea rcx,[rip+..]`. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Value 0x7150f90 on 26.63 (6aed1906-b127418e74d2f2f9)."

  recipe {
    step "pattern" {
      bytes = "0F B6 C0 FF C8 83 F8 ?? 0F 87 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 63 04 81 48 01 C8 FF E0 48 8D 5D ?? 48 89 D9 48 89 FA E8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 4C 89 7D ?? 48 8B 0D"
      offset = 11
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "function_start" {}
}
