entry "LightPropagation.fillSubChunkGrid" {
  kind   = "function"
  hooked = true
  notes  = "dynamic-lighting (Vibrant Visuals hand emitter). The light grid fill of the static light spread: u64 (BlockSource* rcx, int pos[3]* rdx (the SubChunkPos), u16 mode r8w (spilled as a dword to [rsp+70h]; the spread core passes word [cfg+18h]), void* grid[27] r9 (27 SubChunk pointers, index j*9 + ylayer*3 + i), u32* mask [rsp+28h]); returns al = 1 when all 27 subchunks are missing or air (or the centre is missing in mode 100h), so the core skips its block-emitter loop. No LeviLamina name: LeviLamina has only empty LightPropagation classes, so the name is ours. Found by the requester (dynamic-lighting docs/findings/vv-hand-emitter/report.md) and checked here with inspect disasm on every build: the light spread core (the first direct call of LightPropagationCoordinator.propagateForMesh: 26.60 0x5adb870, 26.62 0x5adc030, 26.63 0x5adbd90) empties the thread-local emitter vector (LightPropagation.tlsEmitterVector), then calls this function, then `test al,al; je` to the emitter loop. A second caller exists (26.62 0x5adff95, 26.63 0x5adfcf5), so a hook that only wants the mesher path must filter. Recipe 0 is the body after the pushes and the frame: `mov [rsp+..],r9; mov [rsp+..],r8d; mov rbx,rdx; mov r14,rcx; mov rax,[rip+cookie]; xor rax,rsp; mov [rsp+..],rax; mov eax,[rdx+4]; mov rcx,[rip+..]; test rcx,rcx`, then function_start_of; every displacement is wildcarded. The dword spill of r8 (`44 89 44 24`) is the u16 mode argument. Recipe 1 is the call site in the spread core: ref propagateForMesh, call 0 (the core), then `mov [rsp+..],rax; lea rdx,[rbp+..]; lea r9,[rbp+disp8]; mov rcx,[rbp+..]; call`, read as a branch target; the window has no r8 set-up (the mode is loaded earlier). The two recipes share no bytes. Check: function_start. Values: 26.60 0x5af2020, 26.62 0x5af27e0, 26.63 0x5af2540 (the same body on the three builds). 26.51 not looked at (out of scope; propagateForMesh does not resolve there, so recipe 1 cannot run). What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): refused on purpose, a real code change. The function there (0x41af800, called from the core at 0x4191177) takes another signature: r8 is a pointer (`mov rdi,r8; add rdi,30h`), the grid moved to r8 (`lea r8,[rbp+20h]` at the call), r9 and [rsp+20h] are two u32 pointers, and there is no mode argument. A detour for the 26.6x form would read the grid as the mode, so one name cannot mean both; neither recipe matches (no `mov [rsp+..],r8d` spill; the call site sets r8 between rdx and r9). A Preview hook needs a new entry name."

  recipe {
    step "pattern" { bytes = "4C 89 4C 24 ?? 44 89 44 24 ?? 48 89 D3 49 89 CE 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 84 24 ?? ?? ?? ?? 8B 42 ?? 48 8B 0D ?? ?? ?? ?? 48 85 C9" }
    step "function_start_of" {}
  }

  recipe {
    step "ref" { entry = "LightPropagationCoordinator.propagateForMesh" }
    step "call" { nth = 0 }
    step "pattern" {
      bytes  = "48 89 44 24 ?? 48 8D 95 ?? ?? ?? ?? 4C 8D 4D ?? 48 8B 8D ?? ?? ?? ?? E8"
      offset = 23
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "LightPropagation.tlsEmitterVector" {
  kind  = "field"
  notes = "dynamic-lighting (Vibrant Visuals hand emitter). The offset, in the game's thread-local storage block (gs:[58h][core.tls_index]), of a thread_local std::vector of 16-byte light emitters {Block* @0, u8 x @8, u8 y @9, u8 z @0Ah, u8 emission @0Bh} (begin at the value, end +8, capacity +10h, init flag +18h). The light spread core (the first direct call of LightPropagationCoordinator.propagateForMesh) sets the init flag and registers the TLS destructor on first use, sets end = begin, keeps the vector address in a local, calls LightPropagation.fillSubChunkGrid, and its emitter loop pushes one element per emitting block (`mov [rcx],block; mov [rcx+8..0Bh],x,y,z,emission; add rcx,10h`). Found by the requester (vv-hand-emitter report section 4), checked here with inspect disasm. Recipe 0: ref propagateForMesh, call 0 (the core), then the vector reset `mov eax,[rip+tls_index]; mov rcx,gs:[58h]; mov rcx,[rcx+rax*8]; lea reg,[rcx+X]; mov rax,[rcx+X]; cmp rax,[rcx+X+8]; je`, offset 19 (the lea), operand 1 displacement. The lea's ModRM byte is wildcarded (rdx on 26.6x, rdi on the Preview); every displacement and branch byte is wildcarded; the four bytes of gs:[58h] are kept (literal_ok, fixed by the ABI). Recipe 1: the same core, the init block `mov byte [rax+flag],imm; lea rcx,[rip+dtor]; call` (the only such store-lea-call in the core), operand rip_target of the lea (the TLS destructor of the vector, 26.63 0x5ad7620), then in that destructor `mov rcx,[rax+X]; test rcx,rcx; je` and its displacement. It reads the value in another function from other bytes; it shares only the propagateForMesh-to-core route with recipe 0. Check: range 1000h..2800h. Values: 26.60, 26.62, 26.63 0x18f0. 26.51 not looked at (out of scope; propagateForMesh does not resolve there). What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): the TLS block grew, the vector is at 0x1d78 (init flag 0x1d90); the element is the same 16-byte layout (the push in the Preview core 0x4190d30 writes the pointer at 0, bytes at 8..0Bh, then adds 10h), so the meaning did not change and the entry resolves there. Class: a member that moved; the recipes survived after the ModRM wildcard."

  recipe {
    step "ref" { entry = "LightPropagationCoordinator.propagateForMesh" }
    step "call" { nth = 0 }
    step "pattern" {
      bytes      = "8B 05 ?? ?? ?? ?? 65 48 8B 0C 25 58 00 00 00 48 8B 0C C1 48 8D ?? ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 3B 81 ?? ?? ?? ?? 74"
      offset     = 19
      literal_ok = ["11: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "12: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "13: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "14: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI"]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "LightPropagationCoordinator.propagateForMesh" }
    step "call" { nth = 0 }
    step "pattern" {
      bytes  = "C6 80 ?? ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? E8"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "pattern" {
      bytes  = "48 8B 88 ?? ?? ?? ?? 48 85 C9 74"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 4096
    max = 10240
  }
}
