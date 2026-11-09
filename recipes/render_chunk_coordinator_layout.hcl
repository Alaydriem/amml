entry "RenderChunkCoordinator.vtable" {
  kind  = "global"
  notes = "PoC (dynamic-lighting). The primary vtable of RenderChunkCoordinator (also its BlockSourceListener table). The constructor (Level&, LevelRenderer&, DimensionType) stores it at this+0 and then a float 0.005 at +8. The mod uses it as a guard: a coordinator map node is used only when its object has this vtable. Recipe 0 is the constructor store (`mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; mov dword [rcx+8],3BA3D70Ah`), read as the RIP target of the lea. 1.26.62.01: 0xf328760. See docs/findings/terrain-dirty-path/report.md."

  recipe {
    step "pattern" {
      bytes      = "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 C7 41 ?? 0A D7 A3 3B"
      offset     = 3
      literal_ok = ["16: the float 0.005 the constructor stores at +8 separates this constructor from other vtable stores", "17: as byte 16", "18: as byte 16", "19: as byte 16"]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }
}

entry "LevelRenderer.mRenderChunkCoordinators" {
  kind  = "field"
  notes = "PoC (dynamic-lighting). unordered_map<DimensionType, shared_ptr<RenderChunkCoordinator>> in LevelRenderer: the field holds the list head pointer; each node has the DimensionType at +10h and the coordinator pointer at +18h. The LevelRenderer update (0x53599d0 on 1.26.62.01) walks it and calls the coordinator update for each node (`mov rsi,[rcx+..]; mov rdi,[rsi]; cmp rdi,rsi; je; nop; mov rcx,[rdi+18h]; call`). Read as the displacement of the first mov. 1.26.62.01: 0x30."

  recipe {
    step "pattern" { bytes = "48 8B 71 ?? 48 8B 3E 48 39 F7 74 ?? 66 66 2E 0F 1F 84 00 ?? ?? ?? ?? 48 8B 4F ?? E8" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }
}

entry "LevelRenderer.mPointLightCoordinator" {
  kind  = "field"
  notes = "PoC (dynamic-lighting). The Vibrant Visuals PointLightCoordinator pointer in LevelRenderer: the update loads it as `this` for getAnalyticalLights (`mov rcx,rdi; call [..]; mov rax,[rbp+..]; mov rcx,[rax+..]; lea rdx,[rbp+..]; call`). Read as the displacement of `mov rcx,[rax+..]`. 1.26.62.01: 0x68."

  recipe {
    step "pattern" {
      bytes  = "48 89 F9 FF 15 ?? ?? ?? ?? 48 8B 85 ?? ?? ?? ?? 48 8B 48 ?? 48 8D 95 ?? ?? ?? ?? E8"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }
}
