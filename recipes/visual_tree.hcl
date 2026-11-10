entry "VisualTree.mRootControl" {
  kind     = "field"
  notes    = "Offset in VisualTree of the raw pointer of mRootControl, the std::shared_ptr<UIControl> of the root control of a screen (the pointer, then the control block at +8 from it): 8h on 26.60 and 26.62, right after the ControlScreenAction vtable. The root control's name (UIControl.mName) is the raw screen name, for example small_chest_screen. Recipe 0 is UIScene::getRawScreenName (slot 58 of UIScene.vtable): `mov rsi,rdx; mov rax,[rcx+..]; mov rax,[rax+..]; test rax,rax; je; mov rcx,[rax+..]; test rcx,rcx; je; lock inc dword ptr [rcx+..]; mov r14,[rax+..]; jmp; xor r14d,r14d; mov r15,[rax+..]`, read as the displacement of the load of r15 (the control; the earlier loads take the control block). Recipe 1 is VisualTree's control-by-name lookup (`movaps [rbp+..],xmm0; mov qword ptr [rbp+..],0; mov r8,[rcx+..]; ...; lea rcx,[rbp+..]; call <collector>`), read as the displacement of the load of r8, the root handed to the recursive collector. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) is a build gate: the entry refuses as not for this build on 26.51. Value 8h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "pattern" {
      bytes  = "48 89 D6 48 8B 41 ?? 48 8B 40 ?? 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 48 ?? 48 85 C9 74 ?? F0 FF 41 ?? 4C 8B 70 ?? EB ?? 45 31 F6 4C 8B 78 ?? 4D 85 FF"
      offset = 42
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F 29 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 4C 8B 41 ?? 48 89 45 ?? 48 8B 45 ?? 48 89 45 ?? 48 8B 45 ?? 48 89 45 ?? 48 8D 55 ?? 48 89 55 ?? 48 8D 4D ?? E8"
      offset = 12
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 64
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
