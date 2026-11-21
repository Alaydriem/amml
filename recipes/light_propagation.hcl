entry "LightPropagation.fillSubChunkGrid" {
  kind   = "function"
  hooked = true
  notes  = "dynamic-lighting (Vibrant Visuals hand emitter). The light grid fill of the static light spread: u64 (BlockSource* rcx, int pos[3]* rdx (the SubChunkPos), u16 mode r8w (spilled as a dword to [rsp+70h]; the spread core passes word [cfg+18h]), void* grid[27] r9 (27 SubChunk pointers, index j*9 + ylayer*3 + i), u32* mask [rsp+28h]); returns al = 1 when all 27 subchunks are missing or air (or the centre is missing in mode 100h), so the core skips its block-emitter loop. No LeviLamina name: LeviLamina has only empty LightPropagation classes, so the name is ours. Found by the requester (dynamic-lighting docs/findings/vv-hand-emitter/report.md) and checked here with inspect disasm on every build: the light spread core (the first direct call of LightPropagationCoordinator.propagateForMesh: 26.60 0x5adb870, 26.62 0x5adc030, 26.63 0x5adbd90) empties the thread-local emitter vector (LightPropagation.tlsEmitterVector), then calls this function, then `test al,al; je` to the emitter loop. A second caller exists (26.62 0x5adff95, 26.63 0x5adfcf5), so a hook that only wants the mesher path must filter. Recipe 0 is the body after the pushes and the frame: `mov [rsp+..],r9; mov [rsp+..],r8d; mov rbx,rdx; mov r14,rcx; mov rax,[rip+cookie]; xor rax,rsp; mov [rsp+..],rax; mov eax,[rdx+4]; mov rcx,[rip+..]; test rcx,rcx`, then function_start_of; every displacement is wildcarded. The dword spill of r8 (`44 89 44 24`) is the u16 mode argument. Recipe 1 is the call site in the spread core: ref propagateForMesh, call 0 (the core), then `mov [rsp+..],rax; lea rdx,[rbp+..]; lea r9,[rbp+disp8]; mov rcx,[rbp+..]; call`, read as a branch target; the window has no r8 set-up (the mode is loaded earlier). The two recipes share no bytes. Check: function_start. Values: 26.60 0x5af2020, 26.62 0x5af27e0, 26.63 0x5af2540 (the same body on the three builds). 26.51 not looked at (out of scope; propagateForMesh does not resolve there, so recipe 1 cannot run). What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): refused on purpose, a real code change. The function there (0x41af800, called from the core at 0x4191177) takes another signature: r8 is a pointer (`mov rdi,r8; add rdi,30h`), the grid moved to r8 (`lea r8,[rbp+20h]` at the call), r9 and [rsp+20h] are two u32 pointers, and there is no mode argument. A detour for the 26.6x form would read the grid as the mode, so one name cannot mean both; neither recipe matches (no `mov [rsp+..],r8d` spill; the call site sets r8 between rdx and r9). A Preview hook needs a new entry name. Layout gate: 26.63 only; on 70.x the function takes the 70.23 signature (the grid in r8, a u32 pointer in r9, no mode argument); the replacement is LightPropagation.fillSubChunkGridNoMode."

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
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "LightPropagation.fillSubChunkGridNoMode" {
  kind   = "function"
  hooked = true
  notes  = "dynamic-lighting (Vibrant Visuals hand emitter). The light grid fill of the static light spread on 70.x, the replacement of LightPropagation.fillSubChunkGrid (26.6x), which has another signature. Signature on 70.26: u64 (BlockSource* rcx, int pos[3]* rdx (the SubChunkPos, the y read as a word at [rdx+4]), void* grid[27] r8 (27 SubChunk pointers; the core zeroes 0xD8 bytes for it), u32* r9 (a dword the core zeroes before the call and copies to its frame after it), u32* mask [rsp+28h] (the core stores it to [state+2Ch] after the call, as 26.63 did with its mask)); returns al, and the core goes to its block-emitter loop on `test al,al; je` as on 26.63. There is no mode argument: 26.63 passed a word in r8 and the grid in r9. A 26.63 detour that only passes its five arguments through and reads rdx (the SubChunkPos) after the call still sees the same rdx; one that reads r8 as the mode or r9 as the grid reads the wrong values. The function has two callers, as on 26.63 (the spread core and a second one), so a hook that only wants the mesher path must filter. Recipe 0 is the body after the pushes and the frame: `mov [rsp+..],r9; mov rdi,r8; mov rbx,rdx; mov r14,rcx; mov rax,[rip+cookie]; xor rax,rsp; mov [rsp+..],rax; movsx rsi,word ptr [rdx+4]; call`, then function_start_of; every displacement and the call bytes are wildcards; `mov rdi,r8` (where 26.63 spills r8d as the mode) is the shape of this signature. Recipe 1 is the call site in the spread core: ref LightPropagationCoordinator.propagateForMesh, call 0 (the core), then `mov [rsp+..],rax; lea rdx,[rbp+..]; lea r8,[rbp+disp8]; lea r9,[rbp+disp32]; mov rcx,[rbp+..]; call`, read as a branch target; the r8 set-up between rdx and r9 is the new grid argument. The two recipes share no bytes. Check: function_start. Evidence from the 26.63 function: (1) its call site in the spread core (after the emptying of the thread-local emitter vector, LightPropagation.tlsEmitterVector, and the zeroing of the 27-pointer grid; the 5th argument a zeroed dword; `test al,al; je` after the call) is at the same place in the 70.26 core, after the same vector reset and grid zeroing, and it calls this function; (2) the 26.63 body head (the eight pushes, frame C8h, `mov [rsp+50h],r9; mov rbx,rdx; mov r14,rcx`, the stack cookie, the SubChunkPos y at [rdx+4], then y-1, y+1, y+7FEh, y+7FFh and y+800h with `shl 4`) is the head of this body in the same order, with the same frame size and the same r9 spill slot. Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552). What changed in 6afa4e47-dadc375797a3d31a (70.26): the signature (as on Preview 1.26.70.23): the mode argument is gone, the grid moved from r9 to r8, r9 is a new u32 pointer. Class: real code change."

  recipe {
    step "pattern" { bytes = "4C 89 4C 24 ?? 4C 89 C7 48 89 D3 49 89 CE 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 84 24 ?? ?? ?? ?? 48 0F BF 72 ?? E8" }
    step "function_start_of" {}
  }

  recipe {
    step "ref" { entry = "LightPropagationCoordinator.propagateForMesh" }
    step "call" { nth = 0 }
    step "pattern" {
      bytes  = "48 89 44 24 ?? 48 8D 95 ?? ?? ?? ?? 4C 8D 45 ?? 4C 8D 8D ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? E8"
      offset = 30
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}

entry "LightPropagation.tlsEmitterVector" {
  kind  = "field"
  notes = "dynamic-lighting (Vibrant Visuals hand emitter). The offset, in the game's thread-local storage block (gs:[58h][core.tls_index]), of a thread_local std::vector of 16-byte light emitters {Block* @0, u8 x @8, u8 y @9, u8 z @0Ah, u8 emission @0Bh} (begin at the value, end +8, capacity +10h, init flag +18h). The light spread core (the first direct call of LightPropagationCoordinator.propagateForMesh) sets the init flag and registers the TLS destructor on first use, sets end = begin, keeps the vector address in a local, calls LightPropagation.fillSubChunkGrid, and its emitter loop pushes one element per emitting block (`mov [rcx],block; mov [rcx+8..0Bh],x,y,z,emission; add rcx,10h`). Found by the requester (vv-hand-emitter report section 4), checked here with inspect disasm. Recipe 0: ref propagateForMesh, call 0 (the core), then the vector reset `mov eax,[rip+tls_index]; mov rcx,gs:[58h]; mov rcx,[rcx+rax*8]; lea reg,[rcx+X]; mov rax,[rcx+X]; cmp rax,[rcx+X+8]; je`, offset 19 (the lea), operand 1 displacement. The lea's ModRM byte is wildcarded (rdx on 26.6x, rdi on the Preview); every displacement and branch byte is wildcarded; the four bytes of gs:[58h] are kept (literal_ok, fixed by the ABI). Recipe 1: the same core, the init block `mov byte [rax+flag],imm; lea rcx,[rip+dtor]; call` (the only such store-lea-call in the core), operand rip_target of the lea (the TLS destructor of the vector, 26.63 0x5ad7620), then in that destructor `mov rcx,[rax+X]; test rcx,rcx; je` and its displacement. It reads the value in another function from other bytes; it shares only the propagateForMesh-to-core route with recipe 0. Check: range 1000h..4000h (it was 1000h..2800h until 6afa4e47-dadc375797a3d31a; see the 70.26 paragraph). Values: 26.60, 26.62, 26.63 0x18f0. 26.51 not looked at (out of scope; propagateForMesh does not resolve there). What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): the TLS block grew, the vector is at 0x1d78 (init flag 0x1d90); the element is the same 16-byte layout (the push in the Preview core 0x4190d30 writes the pointer at 0, bytes at 8..0Bh, then adds 10h), so the meaning did not change and the entry resolves there. Class: a member that moved; the recipes survived after the ModRM wildcard. What changed in 6afa4e47-dadc375797a3d31a (70.26): the TLS block grew again, the vector is at 29C0h (26.60 to 26.63 18F0h, Preview 70.23 1D78h), so the old upper bound 2800h refused a value that both recipes give. The code did not change: recipe 0 matches in the 70.26 core with the same instructions (`mov eax,[rip+tls_index]; mov rcx,gs:[58h]; mov rcx,[rcx+rax*8]; lea rdi,[rcx+X]; mov rax,[rcx+X]; cmp rax,[rcx+X+8]; je; mov [rdi+8],rax`, the end set to the begin) and recipe 1 reaches the same value through the TLS destructor, another function. Evidence that the value is right and the bound was too narrow: the two recipes share only the route to the core and read the number from two different functions (the core and the vector destructor), and both give 29C0h; the vector reset in the 70.26 core is the 26.63 sequence with begin at X and end at X+8. The window is 1000h..4000h (16 KB), which holds every known build and about 1.5 times the largest. Class: a member that moved; the recipes survive, the range was the fault."

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
    max = 16384
  }
}
