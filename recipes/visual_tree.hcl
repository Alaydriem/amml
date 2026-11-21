entry "VisualTree.mRootControl" {
  kind     = "field"
  notes    = "Offset in VisualTree of the raw pointer of mRootControl, the std::shared_ptr<UIControl> of the root control of a screen (the pointer, then the control block at +8 from it): 8h on 26.60, 26.62, 26.63 and 70.26, right after the ControlScreenAction vtable. The root control's name (UIControl.mName) is the raw screen name, for example small_chest_screen. Recipe 0 reads the root through UIScene::getRawScreenName's load (slot 58 of UIScene.vtable). On 26.63 the function inlines the read: `mov rsi,rdx; mov rax,[rcx+..]; mov rax,[rax+..]; test rax,rax; je; mov rcx,[rax+..]; test rcx,rcx; je; lock inc dword ptr [rcx+..]; mov r14,[rax+..]; jmp; xor r14d,r14d; mov r15,[rax+..]`, read as the displacement of the load of r15 (the control; the earlier loads take the control block). On 70.26 the function calls a helper chain (ScreenView, then the visual tree getter that returns the root shared_ptr by value); the second alternative is the tail of that getter: the profiler-scope call, `jmp`, `xor eax,eax`, then `mov rcx,[rdi+..]; mov [rsi],rcx; mov [rsi+8],rax` (the raw pointer, then the control block), read as the displacement of the load of rcx. The 70.26 alternative starts 42 bytes before that load (its first two bytes are the end of the previous instruction) so that both alternatives share the one offset. Recipe 1 reads the root in VisualTree's control-by-name lookup. On 26.63 the wrapper loads it inline: `movaps [rbp+..],xmm0; mov qword ptr [rbp+..],0; mov r8,[rcx+..]; ...; lea rcx,[rbp+..]; call <collector>`, read as the displacement of the load of r8, the root handed to the recursive collector. On 70.26 the collector loads the root itself: `xorps xmm0,xmm0; movups [rdi],xmm0; mov qword ptr [rdi+..],0; mov rsi,[rsi+..]; mov qword ptr [rbp+..],0; ...`, read as the displacement of the load of rsi. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. No build gate: builds before 26.63 are not supported. What changed in 6afa4e47-dadc375797a3d31a (70.26): the root is no longer read inline in either function: a getter function and a collector function read it now (real code change in the bodies, same member; a pattern_any with the 26.63 shape and the 70.26 shape in each recipe). Evidence 1: the old getRawScreenName chain (UIScene+40h, ScreenView+50h, tree+8) is the 70.26 getRawScreenName chain (call with [[UIScene+40h]+50h] to the getter whose tail reads tree+8). Evidence 2: the old slot-52 caller of the control-by-name wrapper (`mov rax,[rcx+40h]; mov rcx,[rax+50h]; lea rdx,..; xor r9d,r9d; call`) is at the same place in the 70.26 slot-52 function, whose callee runs into the collector that reads tree+8. Value 8h on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62), 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (70.26)."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 D6 48 8B 41 ?? 48 8B 40 ?? 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 48 ?? 48 85 C9 74 ?? F0 FF 41 ?? 4C 8B 70 ?? EB ?? 45 31 F6 4C 8B 78 ?? 4D 85 FF",
        "24 ?? 48 89 44 24 ?? 48 8D 0D ?? ?? ?? ?? 48 8D 54 24 ?? 4C 8D 4C 24 ?? 41 B8 ?? ?? ?? ?? E8 ?? ?? ?? ?? E9 ?? ?? ?? ?? 31 C0 48 8B 4F ?? 48 89 0E 48 89 46 ??",
      ]
      offset = 42
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "0F 29 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 4C 8B 41 ?? 48 89 45 ?? 48 8B 45 ?? 48 89 45 ?? 48 8B 45 ?? 48 89 45 ?? 48 8D 55 ?? 48 89 55 ?? 48 8D 4D ?? E8",
        "C0 0F 11 07 48 C7 47 ?? ?? ?? ?? ?? 48 8B 76 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 8B 45 ?? 48 8B 48 ?? 48 85 C9",
      ]
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
}
