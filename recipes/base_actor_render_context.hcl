entry "BaseActorRenderContext.ctor" {
  kind     = "function"
  notes    = "BaseActorRenderContext::BaseActorRenderContext(ScreenContext& screenContext, IClientInstance& client, IMinecraftGame& game): rcx the object (a buffer of BaseActorRenderContext.size bytes, 8-byte aligned), rdx the ScreenContext, r8 the client instance, r9 the game; the object is returned in rax. It stores the vtable, the client and game pointers and the item renderer at +58h, and ends with the byte stores at +0B8h..+0BCh. 24 callers build one on the stack; the game itself does so for every inventory item it draws, passing the game from the client's virtual at +270h (ClientInstance.getMinecraftGame). Recipe 0 is the call in the helper that draws one flying item: `mov rdx,[rsi+..]; lea r15,[rbp+..]; mov rcx,r15; mov r8,r14; mov r9,rax; call`, read as the call target. Recipe 1 is the call in the inventory item renderer: `lea r15,[rbp+..]; mov rcx,r15; mov rdx,r14; mov r8,rbx; mov r9,rax; call`, read the same way. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 57E1DA0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 56 ?? 4C 8D 7D ?? 4C 89 F9 4D 89 F0 49 89 C1 E8 ?? ?? ?? ??"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4C 8D 7D ?? 4C 89 F9 4C 89 F2 49 89 D8 49 89 C1 E8 ?? ?? ?? ??"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "BaseActorRenderContext.dtor" {
  kind     = "function"
  notes    = "BaseActorRenderContext::~BaseActorRenderContext() (the complete object destructor, not a deleting one): rcx the object, void. It stores the base vtable, then releases the variant at +0A8h (a switch on its index byte) and the members. 47 callers destroy a stack object after the draw. Both recipes start at a caller that is found by its own prologue and look inside it, so the pattern needs to be unique only in that function. Recipe 0 is in the flying item helper (the function with the two renderGuiItemNew calls): `call <draw>; nop; lea rdx,[rbp+..]; lea r8,[rbp+..]; mov rcx,rsi; call; nop; lea rcx,[rbp+..]; call`, read as the last call target. Recipe 1 is in the inventory item renderer: `lock dec dword ptr [rsi+0Ch]; jne; mov rax,[rsi]; mov rax,[rax+8]; mov rcx,rsi; call [guard]; lea rcx,[rbp+..]; call`, read the same way. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 57E2220h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 44 0F 29 85 ?? ?? ?? ?? 0F 29 BD ?? ?? ?? ?? 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 49 8B" }
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 90 48 8D 55 ?? 4C 8D 85 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 90 48 8D 4D ?? E8 ?? ?? ?? ??"
      offset = 30
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 85 C0 0F 84 ?? ?? ?? ?? 48" }
    step "pattern" {
      bytes  = "F0 FF 4E ?? 75 ?? 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8D 4D ?? E8 ?? ?? ?? ??"
      offset = 26
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "BaseActorRenderContext.size" {
  kind     = "field"
  notes    = "sizeof(BaseActorRenderContext): 0C0h on 26.60 (the LeviLamina header gives the same; the game builds the object in a 0C0h local). The constructor's last stores are `mov dword ptr [r15+0B8h],10101h` and `mov word ptr [r15+0BCh],0`, so the last member ends at 0BEh and the object is rounded up to the 8-byte alignment of its pointers, 0C0h. The recipe reads the displacement of that last store inside BaseActorRenderContext.ctor (0BCh) and adds 4 (the 2-byte store plus the padding to the alignment); a later build that adds a member after it changes the displacement, and one that adds a longer last store breaks the add, so the range check bounds it. A field with one recipe and a range check, as the porting guide has it for a size. Every displacement and immediate byte is a wildcard. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 0C0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "ref" { entry = "BaseActorRenderContext.ctor" }
    step "pattern" { bytes = "66 41 C7 87 ?? ?? ?? ?? ?? ?? ?? ??" }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" { value = 4 }
  }

  check "range" {
    min = 128
    max = 256
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
