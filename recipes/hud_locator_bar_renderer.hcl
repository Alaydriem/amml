entry "HudLocatorBarRenderer.vtable" {
  kind     = "global"
  notes    = "The primary vtable of HudLocatorBarRenderer (the HUD locator bar: the waypoint dots and the up and down arrows). The class has no RTTI. Found from the texture strings that its update (slot 3) loads (`textures/ui/locator_bg`, `locator_arrow_up`, `locator_arrow_down`, loaded only there); that function is held only by slot 3 of this table. The table is stored by the constructor and by the destructor, and no other code refers to it (`inspect xrefs`): 0xF260170 on 26.60 (ctor 0x57447a0, dtor 0x576cce0), 0xEA3F260 on 26.51 (ctor 0x8dc84e0, dtor 0x8df7470). Recipe 0 is the constructor: the two xmm saves of the prologue, the EH state store, then `mov dword ptr [rcx+8],imm (the alpha 1.0); lea rax,[rip+..]; mov [rcx],rax; mov qword ptr [rcx+10h],imm; mov byte ptr [rcx+18h],imm`; it reads the RIP target of the lea. Every displacement and immediate is a wildcard; the store run alone has a second match on both builds, so the prologue saves before it are part of the pattern. Recipe 1 is the destructor: `mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; lea rbx,[rcx+0E0h] (mMeshes); add rcx,imm32 (mMeshes[5]); call` (the first mesh destructor); it reads the RIP target of the lea. Without the mMeshes[5] offset the pattern matches 8 destructors with the same shape, so the two bytes of that immediate are kept: one alternative per build, because mce::Mesh is 260h bytes on 26.60 (0CC0h) and 278h on 26.51 (0D38h). Each alternative matches once on its own build and never on the other. The two recipes are in two functions and share no pattern bytes. The section check keeps the value in .rdata."

  recipe {
    step "pattern" {
      bytes  = "66 0F 7F 7D ?? 0F 29 75 ?? 48 C7 45 ?? ?? ?? ?? ?? C7 41 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 01 48 C7 41 ?? ?? ?? ?? ?? C6 41 ?? ??"
      offset = 24
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 99 ?? ?? ?? ?? 48 81 C1 C0 0C ?? ?? E8",
        "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 99 ?? ?? ?? ?? 48 81 C1 38 0D ?? ?? E8",
      ]
      offset     = 3
      literal_ok = [
        "0.23: the mMeshes[5] offset 0CC0h in `add rcx,imm32` (26.60): 7 other destructors on 26.60 have the same store and call shape",
        "0.24: the mMeshes[5] offset 0CC0h in `add rcx,imm32` (26.60): 7 other destructors on 26.60 have the same store and call shape",
        "1.23: the mMeshes[5] offset 0D38h in `add rcx,imm32` (26.51): 6 other destructors on 26.51 have the same store and call shape",
        "1.24: the mMeshes[5] offset 0D38h in `add rcx,imm32` (26.51): 6 other destructors on 26.51 have the same store and call shape",
      ]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "HudLocatorBarRenderer.render" {
  kind     = "vslot"
  notes    = "Slot of `void HudLocatorBarRenderer::render(MinecraftUIRenderContext&, IClientInstance&, UIControl&, int)` (the 26.60 body reads rcx, rdx, r8, r9 and one stack int) in the HudLocatorBarRenderer.vtable: 16 on 26.51 and 26.60 (0x8dc8e60 and 0x5745690). Slot 5 (the UIRenderContext overload) is a shared `jmp [rax+80h]` thunk to it. The function draws the bar background, then walks the local player's waypoint list (`mov r13,[r13+mClientLocatorBar+10h]`), sorts it by distance, and draws each visible waypoint with one call of HudLocatorBarRenderer.drawIcon and an arrow mesh. Recipe 0 is the argument stores before the drawIcon call (`mov dword ptr [rsp+80h],imm; mov dword ptr [rsp+..],imm` seven times; `lea rdx,[rbp+..]; lea r8,[rbp+..]; call`), then function_start_of and slot_of the HudLocatorBarRenderer.vtable. Recipe 1 is the start of the list walk (`xorps xmm0,xmm0; movaps [rbp+..],xmm0; mov qword ptr [rbp+..],imm; mov r13,[r13+..]; mov rX,[r13+..]; cmp rX,r13; je`, the ModRM bytes of the two list registers wildcarded because 26.51 uses r14 and 26.60 r12), then function_start_of and slot_of. The two anchors are in different parts of the function and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot."

  recipe {
    step "pattern" { bytes = "C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 48 8D 55 ?? 4C 8D 85 ?? ?? ?? ?? E8" }
    step "function_start_of" {}
    step "slot_of" { vtable = "HudLocatorBarRenderer.vtable" }
  }

  recipe {
    step "pattern" { bytes = "0F 57 C0 0F 29 85 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 4D 8B AD ?? ?? ?? ?? 4D 8B ?? ?? 4D 39 ?? 74" }
    step "function_start_of" {}
    step "slot_of" { vtable = "HudLocatorBarRenderer.vtable" }
  }

  check "range" {
    min = 12
    max = 20
  }
}

entry "HudLocatorBarRenderer.drawIcon" {
  kind     = "function"
  hooked   = true
  notes    = "The icon blit that draws one locator bar waypoint: 0x598a9d0 on 26.60. It has exactly one caller in the exe, HudLocatorBarRenderer.render (the call at 0x5746e85 on 26.60). The name is ours; the owner class is not proven (rcx is a renderer member of the BaseActorRenderContext at +60h). Arguments (26.60, checked in the caller and in the body): rcx the renderer, rdx BaseActorRenderContext*, r8 mce::TexturePtr const*, xmm3 x; on the stack [rsp+20h] y, [rsp+28h] u0, [rsp+30h] u1, [rsp+38h] v0, [rsp+40h] v1, [rsp+48h] w, [rsp+50h] h, [rsp+58h] 1.0, [rsp+60h] alpha, [rsp+68h] packed colour 0xAARRGGBB, [rsp+70h] 0xFFFFFFFF, [rsp+78h] 1.0, [rsp+80h] 1.0. It returns void. Recipe 0 is the body after the prologue (`movaps xmm6,xmm3; mov r15,r8 (r12 on 26.51, so the ModRM byte is a wildcard); mov r14,rdx; mov rsi,rcx; mov ebx,[rbp+..] (the colour); mov rax,[rdx+..]; mov rdi,[rax+..]; cmp byte ptr [rdi+..],imm; jne`), then function_start_of. Recipe 1 is the one call in render (the argument stores, `lea rdx,[rbp+..]; lea r8,[rbp+..]; call`), read as a branch target. The two anchors are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Both recipes give 0x598a9d0 on 26.60 and 0x5596cc0 on 26.51."

  recipe {
    step "pattern" { bytes = "0F 28 F3 4D 89 ?? 49 89 D6 48 89 CE 8B 9D ?? ?? ?? ?? 48 8B 42 ?? 48 8B B8 ?? ?? ?? ?? 80 BF ?? ?? ?? ?? ?? 0F 85" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 48 8D 55 ?? 4C 8D 85 ?? ?? ?? ?? E8"
      offset = 78
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
