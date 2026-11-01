entry "NetworkSystem.send" {
  kind      = "function"
  hooked    = true
  msvc_name = "?send@NetworkSystem@@QEAAXAEBVNetworkIdentifier@@AEBVPacket@@W4SubClientId@@@Z"
  notes     = "Address of `void NetworkSystem::send(NetworkIdentifier const& id, Packet const& packet, SubClientId recipientSubId)`: 0x16b0490 in 26.51 (this in rcx, the id in rdx, the packet in r8, the recipient sub-client id in r9b; no return value). LeviLamina gives no in-image address for it, so it comes from the exe. Two functions load the string \"Header Data\"; this one copies the 0xB0-byte NetworkIdentifier, offers the packet to the listeners at this+1B8h.., then serialises the header (packet id, sub-client ids) with that string, writes the packet, walks the connections (NetworkSystem.mConnections) and calls NetworkSystem.sendInternal. Recipe 0 is the header write (`mov ecx,r13d; lea r13,[rbx+..]; movzx ecx,cl; shl ecx,..; shl r14d,..; or r14d,ecx; or r14d,eax; mov rax,[r13+0]; mov rax,[rax+..]; lea r8,[rip+..]; mov rcx,r13; mov edx,r14d; xor r9d,r9d; call [guard]`; the lea is \"Header Data\"), then function_start_of. Recipe 1 is the per-channel listener call before it (`movzx r15d,byte ptr [rsi+..]; cmp r15,..; ja; mov rax,[rbx+r15*8+..]; test rax,rax; je; mov r14,[rax]; mov rax,[rsi]; mov rax,[rax+..]; mov rcx,rsi; call [guard]`), then function_start_of. The two anchors are in different parts of the body and share no pattern bytes; the second \"Header Data\" function matches neither. Every displacement, immediate and branch byte is a wildcard. The check is function_start; both recipes end in function_start_of, so the entry rests on the two agreeing recipes. Values: 0x16b0490 in 26.51, 0x2360a40 in 26.60. What changed in 6ad79ee4-17d51891a1119914: the NetworkSystem members moved by 28h (the listeners at +1E0h, mConnections at +C8h), a `nop` follows each guarded call, and the id is kept in a frame slot instead of rdi; both anchors end at a call and hold no member offsets, so they match unchanged. Evidence on 26.60: the same string, the same two anchors in one function, and that function calls NetworkSystem.sendInternal at its end, as on 26.51."

  recipe {
    step "pattern" {
      bytes = "44 89 E9 4C 8D AB ?? ?? ?? ?? 0F B6 C9 C1 E1 ?? 41 C1 E6 ?? 41 09 CE 41 09 C6 49 8B 45 ?? 48 8B 40 ?? 4C 8D 05 ?? ?? ?? ?? 4C 89 E9 44 89 F2 45 31 C9 FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "44 0F B6 7E ?? 49 83 FF ?? 77 ?? 4A 8B 84 FB ?? ?? ?? ?? 48 85 C0 74 ?? 4C 8B 30 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15"
    }
    step "function_start_of" {}
  }

  check "function_start" {}
}

entry "NetworkSystem.sendInternal" {
  kind      = "function"
  hooked    = true
  msvc_name = "?_sendInternal@NetworkSystem@@QEAAXAEBVNetworkIdentifier@@AEBVPacket@@AEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"
  notes     = "Address of `void NetworkSystem::_sendInternal(NetworkIdentifier const& id, Packet const& packet, std::string const& data)`: 0x16b08c0 in 26.51 (this in rcx, the id in rdx, the packet in r8, the serialised bytes in r9; no return value). It is the serverbound tap: it finds the connection whose mId equals the id (a walk of NetworkSystem.mConnections with an inline compare of the NetworkIdentifier type at +A8h, then a call of the per-type compare), and tail-jumps to slot 1 of that connection's peer (`mov rcx,[conn+118h]`, NetworkConnection.mPeer) with the data, the packet's reliability (+0Ch) and its priority (+28h). Recipe 0 is the only direct call, at the end of NetworkSystem.send (`mov r9,[rbx+..]; mov rcx,rbx; mov rdx,<id>; mov r8,rsi; call`), read as a branch target. It is a pattern_any, scanned inside NetworkSystem.send, one alternative per shape, both with the call at index 17: on 26.51 the id is in rdi (`mov rdx,rdi`, 3 bytes; the alternative starts one byte earlier with a wildcard), on 26.60 it is reloaded from the frame (`mov rdx,[rbp+..]`, 4 bytes). Recipe 1 is the tail of the body (`test rcx,rcx; je; mov r8d,[rdi+..]; mov r9d,[rdi+..]; mov rax,[rcx]; mov rax,[rax+..]; mov r10,[rip+..]`, the peer call), then function_start_of. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Check function_start. Values: 0x16b08c0 in 26.51, 0x23834b0 in 26.60. What changed in 6ad79ee4-17d51891a1119914: mConnections moved from +A0h to +C8h, the walk keeps the vector length in r12 instead of reloading begin and end, and the caller reloads the id from its frame; recipe 1 holds none of these and matches unchanged. Evidence on 26.60: NetworkSystem.send calls it directly at its end (as on 26.51), and the body tail with the peer tail-jump matches once, in that function."

  recipe {
    step "ref" { entry = "NetworkSystem.send" }
    step "pattern_any" {
      patterns = [
        "?? 4C 8B 8B ?? ?? ?? ?? 48 89 D9 48 89 FA 49 89 F0 E8",
        "4C 8B 8B ?? ?? ?? ?? 48 89 D9 48 8B 55 ?? 49 89 F0 E8",
      ]
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 85 C9 74 ?? 44 8B 47 ?? 44 8B 4F ?? 48 8B 01 48 8B 40 ?? 4C 8B 15"
    }
    step "function_start_of" {}
  }

  check "function_start" {}
}

entry "NetworkSystem.isServer" {
  kind      = "function"
  msvc_name = "?isServer@NetworkSystem@@QEBA_NXZ"
  notes     = "Address of `bool NetworkSystem::isServer() const`: 0x16b0300 in 26.51 (this in rcx, the result in al). The body locks a weak_ptr held by a member at this+D8h (+100h on 26.60), tests the object's flag and returns a bool virtual of it (`mov rax,[rcx]; mov rax,[rax+78h]; call [guard]; mov ebx,eax`). It has two direct callers on each build. Recipe 0 is the call in NetworkSystem::_sortAndPacketizeEvents, the only function that loads the string \"NetworkSystem::_sortAndPacketizeEvents\": string_xref on that string, then a pattern_any inside it for the call (`cmp byte ptr [rbp+..],..; jne; mov rcx,[rbp+..]; mov edx,[rbp+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]; mov r15/r14,[rbp+..]; mov rcx,rsi/r15; call`), read as a branch target. One alternative per shape, both with the call at index 46: on 26.51 the guarded call is followed by the load directly (the alternative starts one byte earlier with a wildcard), on 26.60 by a `nop`. Recipe 1 is the other caller (`add rdi,..; jmp; add rdi,..; mov rcx,rdi; call; test al,al; je; mov rcx,[rdi+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`), read as a branch target. The two recipes are in two functions and share no pattern bytes. The body itself is not a recipe: its weak_ptr lock and the virtual call match about ten functions on each build. Every displacement, immediate and branch byte is a wildcard; the REX bytes of the two registers that differ per build are wildcards. Check function_start. Values: 0x16b0300 in 26.51, 0x2383330 in 26.60. What changed in 6ad79ee4-17d51891a1119914: the member moved by 28h and a `nop` follows each guarded call. Evidence on 26.60: both callers, found from the 26.51 call sites, call the same function, and its body has the same shape as on 26.51 (the lock, the flag test, the bool virtual at +78h)."

  recipe {
    step "string_xref" { text = "NetworkSystem::_sortAndPacketizeEvents" }
    step "pattern_any" {
      patterns = [
        "?? 80 BD ?? ?? ?? ?? ?? 75 ?? 48 8B 8D ?? ?? ?? ?? 8B 95 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 4C 8B ?? ?? ?? ?? ?? ?? 89 ?? E8",
        "80 BD ?? ?? ?? ?? ?? 75 ?? 48 8B 8D ?? ?? ?? ?? 8B 95 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 90 4C 8B ?? ?? ?? ?? ?? ?? 89 ?? E8",
      ]
      offset = 46
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 83 C7 ?? EB ?? 48 83 C7 ?? 48 89 F9 E8 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15"
      offset = 13
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "NetworkSystem.mConnections" {
  kind  = "field"
  notes = "Offset of `mConnections` (std::vector<std::unique_ptr<NetworkConnection>>: begin, end, capacity) in NetworkSystem: 0xA0 in 26.51. The element stride is 8 bytes (`mov rcx,[rax+r15*8]` and `sar rcx,3` on 26.51; `mov r14,[r15+rsi*8]` and `sar r12,3` on 26.60). The LeviLamina header sum (+88h) is wrong for these builds; the code is the source. Recipe 0 is the walk in NetworkSystem.sendInternal (the load of begin and end and the empty test), scanned inside that function; it reads the displacement of the begin load (operand 1). Recipe 1 is the same walk in NetworkSystem.send, scanned inside that function. Both are pattern_any, one alternative per shape: 26.51 loads begin into rax and end into rdx and compares them (`mov rax,[rbx+..]; mov rdx,[rbx+..]; cmp rdx,rax; je`), 26.60 loads them into two other registers and subtracts (`mov r15,[r14/rbx+..]; mov r12,[r14/rbx+..]; sub r12,r15; je`). In send the 26.51 alternative is followed by `xor r14d,r14d` and the 26.60 alternative by `mov rax,[rbp+..]`, because the 26.51 shape occurs twice in send. The two recipes are in two functions. Every displacement and branch byte is a wildcard. Range check 128..512. Values: 0xA0 in 26.51, 0xC8 in 26.60. What changed in 6ad79ee4-17d51891a1119914: the members before it grew by 28h (a real layout change), and the walk keeps the length in a register (a code shape change). Evidence for 0xC8 on 26.60: (1) NetworkSystem.sendInternal, found by its call from NetworkSystem.send, reads begin at +C8h and end at +D0h and compares each element's NetworkIdentifier type at +A8h with the id's; (2) NetworkSystem.send does the same walk at +C8h; the members after it (listeners +1B8h to +1E0h, the send buffers) moved by the same 28h."

  recipe {
    step "ref" { entry = "NetworkSystem.sendInternal" }
    step "pattern_any" {
      patterns = [
        "48 8B 83 ?? ?? ?? ?? 48 8B 93 ?? ?? ?? ?? 48 39 C2 0F 84",
        "4D 8B BE ?? ?? ?? ?? 4D 8B A6 ?? ?? ?? ?? 4D 29 FC 0F 84",
      ]
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "NetworkSystem.send" }
    step "pattern_any" {
      patterns = [
        "48 8B 83 ?? ?? ?? ?? 48 8B 93 ?? ?? ?? ?? 48 39 C2 0F 84 ?? ?? ?? ?? 45 31 F6",
        "4C 8B BB ?? ?? ?? ?? 4C 8B A3 ?? ?? ?? ?? 4D 29 FC 0F 84 ?? ?? ?? ?? 48 8B 45",
      ]
      offset = 0
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
}
