entry "Minecraft.mSimTimer" {
  kind  = "field"
  notes = "Offset of `mSimTimer` in Minecraft: E8h in 26.60. The member is a `Timer&` (8 bytes, a pointer): the code loads the Timer* from it and then reads the Timer through it. It is not an inline Timer. In the client, ClientInstance::init (slot 3 of ClientInstance.vtable, 0x658be40) builds a pair {&ClientInstance.mSimTimer (this+E58h), &ClientInstance.mRealTimer (this+ED8h)} and passes it to a std::function factory; the factory lambda (0x8d3120) copies that pair to +40h/+48h of the Minecraft construction arguments and calls the Minecraft constructor (0xb80b30), which stores args+40h at [this+E8h] and args+48h at [this+F0h]. The result goes to ClientInstance.mMinecraft. So on the client, [minecraft+E8h] == clientInstance+E58h. Minecraft::update (0xb82190; called by ClientInstance::update with [this+1D0h] and by the server update lambda 0x1048dc0 with [ServerInstance+B8h], each time right after the two Timer::advanceTime calls) reads mTicks (+4) of [this+E8h] as its sim tick count and folds mOverflowTime (+1Ch) of the same Timer into it; it reads mTicks of [this+F0h] (mRealTimer) as the real-time tick count. The ClientInstance slot-26 leaf (0x659eba0) returns mAlpha (+8) of [[this+1D0h]+E8h], or 0 when mMinecraft is null. Recipe 0 is the constructor store run: `mov qword ptr [r14+disp],imm; mov rax,[rsi+disp]; mov [r14+sim],rax; mov rax,[rsi+disp]; mov [r14+real],rax; movups xmm0,[rsi+disp]; movups [r14+disp],xmm0`; it reads the displacement of operand 0 of the first store (offset 15). Recipe 1 is the overflow fold in Minecraft::update: `mov rcx,[rdi+sim]; movss xmm0,[rcx+disp]; cvttss2si edx,xmm0; cvttps2dq xmm1,xmm0; cvtdq2ps xmm1,xmm1; subss xmm0,xmm1; movss [rcx+disp],xmm0`; it reads the displacement of operand 1 of the first instruction. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Both are unique in 26.60 and agree. Check: range 80h..200h. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties the entry to 26.60: the 26.51 exe was not available when the entry was written, so it refuses as not for this build on 26.51, and must be widened when a later build is added. Value E8h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "49 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 46 ?? 49 89 86 ?? ?? ?? ?? 48 8B 46 ?? 49 89 86 ?? ?? ?? ?? 0F 10 46 ?? 41 0F 11 86"
      offset = 15
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 8B 8F ?? ?? ?? ?? F3 0F 10 41 ?? F3 0F 2C D0 F3 0F 5B C8 0F 5B C9 F3 0F 5C C1 F3 0F 11 41"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 512
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "Minecraft.mRealTimer" {
  kind  = "field"
  notes = "Offset of `mRealTimer` in Minecraft: F0h in 26.60, 8 bytes after Minecraft.mSimTimer. Like mSimTimer it is a `Timer&` (8 bytes, a pointer), not an inline Timer. The Minecraft constructor (0xb80b30) stores construction-argument +48h at [this+F0h]; on the client that argument is &ClientInstance.mRealTimer (clientInstance+ED8h), from the pair that ClientInstance::init passes to the Minecraft factory (see Minecraft.mSimTimer). Minecraft::update (0xb82190) reads mTicks (+4) of [this+F0h] as the count of its real-time tick loop. The recipe is the constructor store run of Minecraft.mSimTimer recipe 0 (`mov qword ptr [r14+disp],imm; mov rax,[rsi+disp]; mov [r14+sim],rax; mov rax,[rsi+disp]; mov [r14+real],rax; movups xmm0,[rsi+disp]; movups [r14+disp],xmm0`); it reads the displacement of operand 0 of the second store (offset 26). Every displacement and immediate byte is a wildcard; the pattern is unique in 26.60. The entry has one recipe and a check (the rule for a field): range 80h..200h. The gate LevelChunk.mSubChunks = 456 ties it to 26.60 (the 26.51 exe was not available; widen it when a later build is added). Value F0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "49 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 46 ?? 49 89 86 ?? ?? ?? ?? 48 8B 46 ?? 49 89 86 ?? ?? ?? ?? 0F 10 46 ?? 41 0F 11 86"
      offset = 26
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 512
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
