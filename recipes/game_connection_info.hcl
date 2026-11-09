entry "GameConnectionInfo.size" {
  kind     = "field"
  notes    = "sizeof(Social::GameConnectionInfo): 310h in 26.51. The minimap reads the head of the value (mType at 0, mHostIpAddress at 8, mPort at 8Ch, the same on 26.51 and 26.60) through its `#[repr(C)]` mirror GameConnectionInfoLayout, which ends at 90h; this entry is the one value of the layout that moves. The size is also the offset of the engaged flag of std::optional<GameConnectionInfo> (the value the mod reads). LeviLamina header sum: mServerIdentityVerifier 16 at 300h ends at 310h, class alignment 8. Recipe 0 is a getter of another class (a this-adjusting thunk: `add rcx,-728h`, a weak-pointer lock, a virtual call that returns a GameConnectionInfo const&) that copy-constructs into the out optional and then sets its flag: `mov rcx,rsi; mov rdx,rdi; call <copy ctor>; mov byte ptr [rsi+310h],1; mov rax,rsi; epilogue`. Recipe 1 goes through ClientInstance.vtable slot 11 (ClientInstance.getGameConnectionInfo) and reads the first instruction after `mov rax,rdx` in that body, `mov byte ptr [rdx+310h],0`. The slot number in recipe 1 is fixed: if the slot moves, the pattern does not match in the other function and the recipe fails; it never gives a value from the wrong function without the bytes. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard (the flag values 0 and 1 too). What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the size is 310h on 26.51 and 2F0h on 26.60 (the layout after mPort at 8Ch shrank by 20h). Recipe 0 (the optional constructor) still matched and gives 2F0h. Recipe 1 used a fixed vtable_slot 11 for getGameConnectionInfo, which moved to slot 10, so it read another function and found no match (it refused). Recipe 1 is now the getGameConnectionInfo body (`mov rax,rdx; mov byte ptr [rdx+size],0; cmp byte ptr [rcx+9E8h],1; jne`), with the two bytes of the member offset 9E8h kept by literal_ok (three other optional getters have the same body with other members); it reads the size from the first store. Evidence on 26.60: the optional constructor (recipe 0) and the getGameConnectionInfo body (old slot 11 body, found in slot 10) both set the engaged flag at 2F0h; a third function that copies a GameConnectionInfo and then clears the next member writes it at 2F0h (310h on 26.51). Class: real code change (layout). The Rust code assumed 310h: see the Task 8 repair ledger."

  recipe {
    step "pattern" {
      bytes  = "48 89 F1 48 89 FA E8 ?? ?? ?? ?? C6 86 ?? ?? ?? ?? ?? 48 89 F0 48 83 C4 ?? 5B 5F 5E 5D C3"
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes      = "48 89 D0 C6 82 ?? ?? ?? ?? ?? 80 B9 E8 09 ?? ?? ?? 75"
      offset     = 3
      literal_ok = [
        "12: cmp byte ptr [rcx+9E8h],1, the engaged flag of ClientInstance's optional GameConnectionInfo: three other optional getters have the same body with other members (150h, 1D90h, 0CE8h) (26.51 and 26.60)",
        "13: the high byte of the same member offset 9E8h (26.51 and 26.60)",
      ]
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 512
    max = 2048
  }
}

entry "GameConnectionInfo.dtor" {
  kind     = "function"
  notes    = "Social::GameConnectionInfo::~GameConnectionInfo(): 0xc6a80 in 26.51. Shape `(this) -> void`, non-virtual, this in rcx; nothing else is read (the prologue is `push rsi; push rdi; sub rsp,28h; mov rsi,rcx`, and rdx is never used, so it is not a deleting destructor with a flags argument). It does not free `this`: the only calls are the control-block release of the shared_ptr at +300h (`mov rdi,[rcx+308h]; lock dec [rdi+8]`, the virtual dispose and destroy calls), the two member destructors (+198h when the engaged flag at +2C8h is 1, +0D0h), and the operator-delete helper on the std::string heap buffers (rcx = the string's own pointer, never rsi); the function ends with `add rsp,28h; pop rdi; pop rsi; ret`. It frees the strings with the large-string test (`cmp rax,10h; jb`) and the 1000h aligned-block path at +2D8h (std::optional<std::string>, flag +2F8h), +68h, +48h, +28h and +8h (capacity at +80h, +60h, +40h, +20h), in reverse member order. The 26.51 size is 310h (GameConnectionInfo.size) and the shared_ptr is the last member (mServerIdentityVerifier at 300h). Recipe 0 is the body, from the shared_ptr release to the first optional string: `lock dec [rdi+8]; jne; mov rax,[rdi]; mov rax,[rax+8]; mov rcx,rdi; call [guard]; cmp byte ptr [rsi+flag],1; jne; mov rax,[rsi+cap]; cmp rax,10h; jb; mov rcx,[rsi+ptr]; ...; mov qword ptr [rsi+size],0; mov qword ptr [rsi+cap],0Fh; mov byte ptr [rsi+ptr],0; cmp byte ptr [rsi+flag2],1; jne; lea rcx,[rsi+..]`, then function_start_of. It is long on purpose: another destructor with a shared_ptr and an optional string (0xc8f760) matches the first part, and two other functions with optional strings match the shorter tails; only the run through the `mov byte ptr [rsi+..],0; cmp byte ptr [rsi+..],1; jne; lea rcx,[rsi+..]` pair is unique. Recipe 1 is a direct call: in the function at 0xd7510 (it walks an array of 310h-byte GameConnectionInfo elements, `add rdx,310h`, and calls the copy constructor 0xd7690 on its out object), `cmp word ptr [rsi],0FFFFh; jne; mov rcx,rsi; call <dtor>; mov rdx,[rdi]; mov rax,[rdi+8]`, read as a branch target. The recipes share no anchor bytes: one is the body of the callee, the other the bytes around one of its 206 call sites. Every displacement, immediate and branch byte is a wildcard. Check function_start. Grade: two recipes agree and the argument shape was read from the body and the callers (no LeviLamina address was used). Not hooked: the minimap mod calls it on its own copy of the object."

  recipe {
    step "pattern" {
      bytes = "F0 FF 4F ?? 75 ?? 48 8B 07 48 8B 40 ?? 48 89 F9 FF 15 ?? ?? ?? ?? 80 BE ?? ?? ?? ?? ?? 75 ?? 48 8B 86 ?? ?? ?? ?? 48 83 F8 ?? 72 ?? 48 8B 8E ?? ?? ?? ?? 48 8D 50 ?? 48 81 FA ?? ?? ?? ?? 72 ?? 4C 8B 41 ?? 48 83 C1 ?? 4C 29 C1 48 83 F9 ?? 0F 83 ?? ?? ?? ?? 48 83 C0 ?? 48 89 C2 4C 89 C1 E8 ?? ?? ?? ?? 48 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? C6 86 ?? ?? ?? ?? ?? 80 BE ?? ?? ?? ?? ?? 75 ?? 48 8D 8E"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "66 83 3E ?? 75 ?? 48 89 F1 E8 ?? ?? ?? ?? 48 8B 17 48 8B 47 ?? 90 48 39 C2 74"
      offset = 9
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "GameConnectionInfo.ctor" {
  kind      = "function"
  msvc_name = "??0GameConnectionInfo@Social@@QEAA@W4ConnectionType@1@W4TransportLayer@@AEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@H@Z"
  notes     = "Social::GameConnectionInfo::GameConnectionInfo(ConnectionType type, TransportLayer transport, std::string const& ip, int port): 0xd76730 in 26.51, 0x4ec610 in 26.60. Arguments: rcx = the object (GameConnectionInfo.size bytes), dx = the type (a u16 at +0), r8d = the transport (+4), r9 = the ip string (copied to mHostIpAddress at +8), and one stack argument at entry+28h, the port (stored at +8Ch). It replaces a move constructor: a caller builds the object in its own buffer with this function and passes the buffer as a by-value argument (for example to ClientInstance.startExternalNetworkWorld, which destroys it). Four sibling constructors start with the same stores (`mov rsi,r9; mov r15,rcx; mov word ptr [rcx],dx; mov dword ptr [rcx+4],r8d; ...`); they read two or three stack arguments and have another frame (the five-argument form has no xmm6 save and a smaller `sub rsp`). Only this one saves xmm6 with this push list and reads one stack argument. Recipe 0 is its prologue and first stores: the seven pushes, `sub rsp,imm32; lea rbp,[rsp+..]; movaps [rbp+..],xmm6; mov qword ptr [rbp+..],-2`, then `mov rsi,r9; mov r15,rcx; mov [rcx],dx; mov [rcx+4],r8d; xorps; movups [rcx+..] (two); mov rdi,[r9+..]; cmp qword ptr [r9+..],imm; jb` at offset 0. Recipe 1 is a direct call in a function that connects to the local host (0x8921e0 in 26.51, 0x94b2b0 in 26.60): it writes \"127.0.0.1\" into a stack string (`movabs rcx,imm64; mov [rbp+..],rcx; mov byte ptr [rbp+..],imm`), then `movzx eax,ax; mov byte ptr [rbp+..],imm; mov [rsp+20h],eax` (the port), `lea rcx,[rbp+..]; lea r9,[rbp+..]; xor edx,edx; mov r8d,imm; call`; it reads the branch target at offset 57. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard (the text bytes too). Both are unique in both exes and agree. Check: function_start. Not hooked: a mod calls it. What changed in 26.60: the address only; the code is the same, and the members after +8Ch moved (see GameConnectionInfo.size)."

  recipe {
    step "pattern" {
      bytes = "55 41 57 41 56 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 0F 29 75 ?? 48 C7 45 ?? ?? ?? ?? ?? 4C 89 CE 49 89 CF 66 89 11 44 89 41 ?? 0F 57 C0 0F 11 41 ?? 0F 11 41 ?? 49 8B 79 ?? 49 83 79 ?? ?? 72"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 B9 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 8D ?? ?? ?? ?? C6 85 ?? ?? ?? ?? ?? 0F B7 C0 C6 85 ?? ?? ?? ?? ?? 89 44 24 ?? 48 8D 4D ?? 4C 8D 8D ?? ?? ?? ?? 31 D2 41 B8 ?? ?? ?? ?? E8"
      offset = 57
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "GameConnectionInfo.mUnresolvedUrl" {
  kind  = "field"
  notes = "Social::GameConnectionInfo::mUnresolvedUrl, a std::string (32 bytes): 28h on 26.51, 26.60, 26.62 and 26.63. It holds the host text before name resolution: a runtime tap of the Play > Servers join on 26.63 read `bedrock-voice-chat.nodecraft.gg` here while mHostIpAddress (+8h) held the resolved `79.127.172.199`. The name is the LeviLamina member name (src/mc/network/GameConnectionInfo.h): mType 0, mTransportType 4, mHostIpAddress 8, mUnresolvedUrl 28h, mServerRegion 48h, mExternalServerName 68h, mServiceQuality 88h, mPort 8Ch. The `#[repr(C)]` mirror GameConnectionInfoLayout calls it `url` at 28h. How it was found: the four constructors that start with `mov [rcx],dx; mov [rcx+4],r8d` (type, transport) and copy the ip string to +8h were read on 26.63 (0x4ec4f0, 0x4ec8b0, 0x4ecf00, 0x4ed380). The constructor (type, transport, ip, port, std::string& serverRegion, int serviceQuality) at 0x4ed380 copies its serverRegion argument to +48h and its serviceQuality to +88h, and the one with an externalServerName argument (0x4ecf00) copies that argument to +68h (see GameConnectionInfo.mExternalServerName); the string member between mHostIpAddress (+8h) and mServerRegion (+48h) is mUnresolvedUrl at 28h, which no constructor fills. Recipe 0 is inside GameConnectionInfo.ctor (type, transport, ip, port): right after the ip copy, `mov eax,[rbp+..] (the port argument); lea rcx,[r15+..]; mov [rbp+..],rcx; xorps xmm6,xmm6; movups [r15+28h],xmm6`, the first of the three empty-string inits that follow mHostIpAddress; it reads the store displacement at offset 14. Recipe 1 is in the serverRegion constructor (0xd775e0 in 26.51, 0x4ed4a0 in 26.60 and 26.62, 0x4ed380 in 26.63): `mov rsi,[rbp+..] (the serverRegion argument); xorps xmm0,xmm0; movups [r12+28h],xmm0; mov qword ptr [r12+38h],0; mov qword ptr [r12+40h],0Fh; movups [r12+48h],xmm0; movups [r12+58h],xmm0; mov rdi,[rsi+10h]; cmp qword ptr [rsi+18h],10h; jb`: the empty string just before the string that receives a copy of the argument (two movups instead of size and capacity stores). It reads the first store displacement at offset 7. The six-argument externalServerName constructor has one more empty init in that run and does not match. The two recipes are in two functions and share no bytes. Every displacement and immediate byte is a wildcard; both patterns are unique in all four exes. Check: range 10h..80h (after mHostIpAddress, before mPort). Not a hook; a mod writes the host text here before it passes the info to the join call."

  recipe {
    step "ref" { entry = "GameConnectionInfo.ctor" }
    step "pattern" {
      bytes  = "8B 45 ?? 49 8D 4F ?? 48 89 4D ?? 0F 57 F6 41 0F 11 77 ??"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 75 ?? 0F 57 C0 41 0F 11 44 24 ?? 49 C7 44 24 ?? ?? ?? ?? ?? 49 C7 44 24 ?? ?? ?? ?? ?? 41 0F 11 44 24 ?? 41 0F 11 44 24 ?? 48 8B 7E ?? 48 83 7E ?? ?? 72"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 128
  }
}

entry "GameConnectionInfo.mExternalServerName" {
  kind  = "field"
  notes = "Social::GameConnectionInfo::mExternalServerName, a std::string (32 bytes): 68h on 26.51, 26.60, 26.62 and 26.63. It holds the server list name: a runtime tap of the Play > Servers join on 26.63 read `Nodecraft` here. The name is the LeviLamina member name (src/mc/network/GameConnectionInfo.h; the order is mHostIpAddress 8, mUnresolvedUrl 28h, mServerRegion 48h, mExternalServerName 68h, mServiceQuality 88h, mPort 8Ch). How it was found: LeviLamina lists a constructor (type, transport, std::string const& ip, int port, ThirdPartyInfo const& serverInfo, std::string const& externalServerName). On 26.63 it is 0x4ecf00 (0xd77160 in 26.51, 0x4ed020 in 26.60 and 26.62): it copies ip to +8h, stores the port at +8Ch, copy-constructs the ThirdPartyInfo argument at +0D0h and copies its last stack argument (externalServerName) to +68h. A second, independent fact: the serverRegion constructor (0x4ed380 in 26.63) copies serverRegion to +48h and serviceQuality to +88h, and inits the one string member between them at +68h. Recipe 0 is in the externalServerName constructor: `mov rsi,[rbp+..] (the externalServerName argument); xorps xmm0,xmm0; movups [r12+28h],xmm0; size and capacity stores; movups [r12+48h],xmm0; size and capacity stores; movups [r12+68h],xmm0; movups [r12+78h],xmm0; mov rdi,[rsi+10h]; cmp qword ptr [rsi+18h],10h; jb`: the string zeroed with two movups is the one that then receives the copy of the argument. It reads the store displacement at offset 58. Recipe 1 is in the serverRegion constructor after the region copy (the pattern starts after that call): `mov eax,[rbp+..] (serviceQuality); mov ecx,[rbp+..] (port); lea rdx,[r12+..]; mov [rbp+..],rdx; xorps xmm6,xmm6; movups [r12+68h],xmm6; mov qword ptr [r12+78h],0; mov qword ptr [r12+80h],0Fh; mov [r12+88h],eax; mov [r12+8Ch],ecx`: the empty string just before mServiceQuality and mPort. It reads the store displacement at offset 21. The two recipes are in two functions and share no bytes. Every displacement and immediate byte is a wildcard; both patterns are unique in all four exes. Check: range 10h..88h (after mHostIpAddress, before mServiceQuality). Not a hook; a mod writes the server name here before it passes the info to the join call."

  recipe {
    step "pattern" {
      bytes  = "48 8B B5 ?? ?? ?? ?? 0F 57 C0 41 0F 11 44 24 ?? 49 C7 44 24 ?? ?? ?? ?? ?? 49 C7 44 24 ?? ?? ?? ?? ?? 41 0F 11 44 24 ?? 49 C7 44 24 ?? ?? ?? ?? ?? 49 C7 44 24 ?? ?? ?? ?? ?? 41 0F 11 44 24 ?? 41 0F 11 44 24 ?? 48 8B 7E ?? 48 83 7E ?? ?? 72"
      offset = 58
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "8B 85 ?? ?? ?? ?? 8B 4D ?? 49 8D 54 24 ?? 48 89 55 ?? 0F 57 F6 41 0F 11 74 24 ?? 49 C7 44 24 ?? ?? ?? ?? ?? 49 C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? 41 89 84 24 ?? ?? ?? ?? 41 89 8C 24"
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 136
  }
}
