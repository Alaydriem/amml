entry "GuiMessage.ctor" {
  kind     = "function"
  hooked   = true
  notes    = "GuiMessage::GuiMessage(MessageType type, std::string const& username, std::string const& message, std::optional<std::string> filteredMessage, std::string const& ttsMessage, std::string const& authorXuid, float lifetime, bool forceVisible, bool ttsRequired): rcx = this, edx = type, r8 = username, r9 = message, [rsp+28h] = a pointer to the filteredMessage temporary (passed by value: the callee destroys it, so a hook must not read it after the original returns), [rsp+30h] = ttsMessage, [rsp+38h] = authorXuid, [rsp+40h] = lifetime, [rsp+48h] = forceVisible, [rsp+50h] = ttsRequired; it returns this in rax. It copies the arguments to mType +0h, mMessage +8h, mFilteredMessage +28h (optional, engaged byte +48h), mTTSMessage +50h, mUsername +70h, mAuthorXuid +D8h, forceVisible +F8h, ttsRequired +F9h, lifetime +FCh, then builds mFullString +90h (`<username> message`, or the message alone when the username is empty) and mFilteredFullString +B0h (optional, engaged byte +D0h). sizeof(GuiMessage) is 110h. Found from the TextPacket handler (slot 244 of ClientNetworkHandler.vtable): each GuiData display function calls GuiData::_addMessage (0xe8e420 on 26.60), which calls this constructor once (0xe8e9d5); the only other caller (0xe8cea0) makes type 9 messages for another list. Recipe 0 is the body after the last argument copy (`movss xmm0,[rbp+..]; movzx eax,byte ptr [rbp+..]; movzx ecx,byte ptr [rbp+..]; mov [rsi+..],cl; mov [rsi+..],al; movss [rsi+..],xmm0; mov word ptr [rsi+..],0; mov dword ptr [rsi+..],0; mov byte ptr [rsi+..],0`), then function_start_of. Recipe 1 is the call in GuiData::_addMessage: the stack argument stores and the register setup (`lea r8,[r9+..]; mov edx,[r9]; mov [rsp+..],al; mov [rsp+..],cl; movss [rsp+..],xmm0; mov rax,[rbp+..]; mov [rsp+..],rax; lea rax,[rbp+..]; mov [rsp+..],rax; lea rax,[rbp+..]; mov [rsp+..],rax; lea rcx,[rbp+..]; lea r9,[rbp+..]; call`), read as a branch target. They are in two functions. Every displacement, immediate and branch byte is a wildcard. In 6ad79ee4-17d51891a1119914 (26.60) the constructor is at 0xE8D420. The recipes were written on 26.60 only; 26.51 was not checked. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the constructor takes one more stack argument (a 2-byte value at [rsp+58h]; the lifetime stays at [rsp+40h] and the two flag bytes stay at [rsp+48h] and [rsp+50h]; slots counted from the stack pointer on entry, the return address at [rsp+0]; evidence: the 70.26 constructor reads [rbp+100h] as a word, [rbp+0E8h] as a float and [rbp+0F0h], [rbp+0F8h] as bytes, with rbp = entry rsp - 0A8h) and stores its flags differently at the end (a word at +F8h, bytes at +FAh and +FBh, the clear of the last members at other offsets). Recipe 0 is a pattern_any: the second alternative is the new end of the body. Recipe 1 is a pattern_any: the second alternative is the new call set-up (one more store of a byte; it starts with six wildcard bytes so that the call is at offset 71 in both). Evidence: (1) the first 100 bytes of the body (`mov r15,r9; mov [rbp+..],r8; mov rsi,rcx; mov [rcx],edx; xorps; movups [rcx+..],xmm0 twice; mov r14,[r9+..]; cmp qword ptr [r9+..],..`) match once in the old constructor and once in the new one, and the new one has two callers as the old one has; (2) the call set-up recipe reaches the same function. Class: real code change (a new argument and a changed member layout: mod-code work). One name, one layout: this entry means the 26.6x argument list only (check entry_is GuiMessage.ctorShape = 0), so it is refused as not for this build on 70.x, where GuiMessage.ctorWithFlagWord is the constructor."

  recipe {
    step "pattern_any" {
      patterns = [
        "F3 0F 10 85 ?? ?? ?? ?? 0F B6 85 ?? ?? ?? ?? 0F B6 8D ?? ?? ?? ?? 88 8E ?? ?? ?? ?? 88 86 ?? ?? ?? ?? F3 0F 11 86 ?? ?? ?? ?? 66 C7 86 ?? ?? ?? ?? ?? ?? C7 86 ?? ?? ?? ?? ?? ?? ?? ?? C6 86 ?? ?? ?? ?? ??",
        "0F B7 85 ?? ?? ?? ?? F3 0F 10 85 ?? ?? ?? ?? 0F B6 8D ?? ?? ?? ?? 0F B6 95 ?? ?? ?? ?? 66 89 86 ?? ?? ?? ?? 88 96 ?? ?? ?? ?? 88 8E ?? ?? ?? ?? F3 0F 11 86 ?? ?? ?? ?? 66 C7 86 ?? ?? ?? ?? ?? ?? C7 86 ?? ?? ?? ?? ?? ?? ?? ??",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "4D 8D 41 ?? 41 8B 11 88 44 24 ?? 88 4C 24 ?? F3 0F 11 44 24 ?? 48 8B 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8",
        "?? ?? ?? ?? ?? ?? 88 4C 24 ?? 44 88 4C 24 ?? F3 0F 11 44 24 ?? 48 8B 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8",
      ]
      offset = 71
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "entry_is" {
    entry = "GuiMessage.ctorShape"
    value = 0
  }

  check "function_start" {}
}

entry "GuiMessage.ctorShape" {
  kind     = "field"
  notes    = "Which argument list GuiMessage::GuiMessage has: 0 on the 26.6x shape (the last argument at [rsp+50h]: type, username, message, filtered message, tts message, author xuid, lifetime, forceVisible, ttsRequired; the flag bytes are stored at +F8h and +F9h), 1 on the 70.x shape (one more argument, a 2-byte value at [rsp+58h] that the constructor stores as a word at +F8h, and the two bool arguments stored at +FAh and +FBh). 0 on 6aed1906-b127418e74d2f2f9 (26.63), 1 on 6afa4e47-dadc375797a3d31a (70.26). GuiMessage.ctorWithFlagWord resolves only on 1 (entry_is). Both recipes are the patterns of the GuiMessage.ctor recipes over the executable: recipe 0 is the end of the constructor body (alternative 0 the 26.6x flag stores, alternative 1 the 70.x flag stores with the extra word), then the `alternative` step; recipe 1 is the call set-up in GuiData::_addMessage (alternative 0 the 26.6x stores, alternative 1 the 70.x set-up with one more byte store), then `alternative`. Each alternative matches once on its own build and never on the other. Every displacement, immediate and branch byte is a wildcard. The check is range 0..1. Class: real code change (a new argument). Evidence: the decompile of both constructors shows one more parameter (undefined2) on 70.26, stored with `mov word ptr [this+F8h]`, and the signature probe reports the extra stack argument at [rsp+58h]. GuiMessage.ctor resolves only on 0 (entry_is), so a mod cannot hook the 26.6x argument list on 70.x."

  recipe {
    step "pattern_any" {
      patterns = [
        "F3 0F 10 85 ?? ?? ?? ?? 0F B6 85 ?? ?? ?? ?? 0F B6 8D ?? ?? ?? ?? 88 8E ?? ?? ?? ?? 88 86 ?? ?? ?? ?? F3 0F 11 86 ?? ?? ?? ?? 66 C7 86 ?? ?? ?? ?? ?? ?? C7 86 ?? ?? ?? ?? ?? ?? ?? ?? C6 86 ?? ?? ?? ?? ??",
        "0F B7 85 ?? ?? ?? ?? F3 0F 10 85 ?? ?? ?? ?? 0F B6 8D ?? ?? ?? ?? 0F B6 95 ?? ?? ?? ?? 66 89 86 ?? ?? ?? ?? 88 96 ?? ?? ?? ?? 88 8E ?? ?? ?? ?? F3 0F 11 86 ?? ?? ?? ?? 66 C7 86 ?? ?? ?? ?? ?? ?? C7 86 ?? ?? ?? ?? ?? ?? ?? ??",
      ]
    }
    step "alternative" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "4D 8D 41 ?? 41 8B 11 88 44 24 ?? 88 4C 24 ?? F3 0F 11 44 24 ?? 48 8B 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8",
        "?? ?? ?? ?? ?? ?? 88 4C 24 ?? 44 88 4C 24 ?? F3 0F 11 44 24 ?? 48 8B 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8",
      ]
    }
    step "alternative" {}
  }

  check "range" {
    min = 0
    max = 1
  }
}

entry "GuiMessage.ctorWithFlagWord" {
  kind     = "function"
  hooked   = true
  notes    = "GuiMessage::GuiMessage on the 70.x argument list (GuiMessage.ctorShape = 1): rcx = this, edx = type, r8 = username, r9 = message, [rsp+28h] = a pointer to the filteredMessage temporary, [rsp+30h] = ttsMessage, [rsp+38h] = authorXuid, [rsp+40h] = lifetime (float), [rsp+48h] = forceVisible (bool), [rsp+50h] = ttsRequired (bool), [rsp+58h] = a 2-byte value that the constructor stores as a word at +F8h (the name is ours: the decompile shows an undefined2 parameter that is only stored there). It returns this in rax. One name, one layout: this entry means the 70.x list only (check entry_is GuiMessage.ctorShape = 1), so it is refused as not for this build on 26.63. The recipes are those of GuiMessage.ctor (see its notes for the anchors and the 70.26 evidence): recipe 0 the end of the body (the second alternative of its pattern_any is the 70.x shape), then function_start_of; recipe 1 the call in GuiData::_addMessage (the second alternative is the 70.x set-up), read as the call target. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Value: the same function as GuiMessage.ctor on 6afa4e47-dadc375797a3d31a (70.26). Class: new entry (the signature change of 26.70.26). Evidence: the decompile of the 70.26 constructor (parameters this, int, string, string, optional string, string, string, float, bool, bool, undefined2) and the signature probe (extra stack argument at [rsp+58h]) agree."

  recipe {
    step "pattern_any" {
      patterns = [
        "F3 0F 10 85 ?? ?? ?? ?? 0F B6 85 ?? ?? ?? ?? 0F B6 8D ?? ?? ?? ?? 88 8E ?? ?? ?? ?? 88 86 ?? ?? ?? ?? F3 0F 11 86 ?? ?? ?? ?? 66 C7 86 ?? ?? ?? ?? ?? ?? C7 86 ?? ?? ?? ?? ?? ?? ?? ?? C6 86 ?? ?? ?? ?? ??",
        "0F B7 85 ?? ?? ?? ?? F3 0F 10 85 ?? ?? ?? ?? 0F B6 8D ?? ?? ?? ?? 0F B6 95 ?? ?? ?? ?? 66 89 86 ?? ?? ?? ?? 88 96 ?? ?? ?? ?? 88 8E ?? ?? ?? ?? F3 0F 11 86 ?? ?? ?? ?? 66 C7 86 ?? ?? ?? ?? ?? ?? C7 86 ?? ?? ?? ?? ?? ?? ?? ??",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "4D 8D 41 ?? 41 8B 11 88 44 24 ?? 88 4C 24 ?? F3 0F 11 44 24 ?? 48 8B 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8",
        "?? ?? ?? ?? ?? ?? 88 4C 24 ?? 44 88 4C 24 ?? F3 0F 11 44 24 ?? 48 8B 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8",
      ]
      offset = 71
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "entry_is" {
    entry = "GuiMessage.ctorShape"
    value = 1
  }

  check "function_start" {}
}

entry "GuiMessage.size" {
  kind     = "field"
  notes    = "sizeof(GuiMessage), the stride of the message list that GuiData keeps (a vector of GuiMessage by value: begin and end pointers at GuiData +168h and +170h on 26.63, +190h and +198h on 70.26): 110h on 6aed1906-b127418e74d2f2f9 (26.63), 118h on 6afa4e47-dadc375797a3d31a (70.26). The size grows by 8 on 70.26: the constructor stores one more word at +F8h, the two flags at +FAh and +FBh, the lifetime at +FCh (unchanged), and clears the tail through +111h (a qword store at +109h) where 26.63 ends at +109h. It is the size check of the GuiMessage mirror: a mirror selects the layout by this value (110h or 118h) and a value that no mirror has refuses. Recipe 0 is the expiry scan of the message list (the GuiData function that handles messages whose lifetime ran out): `movss xmm1,[<element>+0FCh]; ucomiss xmm1,xmm0; jbe; add <element>,imm32; cmp; jne`, read as the immediate of the add (the lifetime read at FCh is the same on both builds, which tells this vector apart from other vectors). Recipe 1 is the next loop of the same function: `cmp; je; lea r13,[r15+imm32]; cmp r13,r12; je; xorps xmm6,xmm6; jmp short; add r13,imm32`, read as the displacement of the lea. The two recipes are two loops of the same function and share no pattern bytes. Every register, displacement, immediate and branch byte is a wildcard. The range check is a window around both values. Evidence: on both builds every step of those loops (`add r15,..`, `add r12,..`, `add r13,..` and the lea) uses the same stride, 110h or 118h."

  recipe {
    step "pattern" {
      bytes  = "F3 ?? 0F 10 ?? ?? ?? ?? ?? 0F 2E ?? 76 ?? ?? 81 ?? ?? ?? ?? ?? ?? 39 ?? 75 ??"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "?? 39 ?? 74 ?? ?? 8D ?? ?? ?? ?? ?? ?? 39 ?? 74 ?? 0F 57 ?? EB ?? ?? 81 ?? ?? ?? ?? ??"
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}

entry "GuiMessage.mType" {
  kind     = "field"
  notes    = "Offset in GuiMessage of mType, the MessageType enum (4 bytes): 0 on 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (70.26). The constructor stores its second argument first: `mov r15,r9; mov [rbp+..],r8; mov rsi,rcx; mov [rcx],edx; xorps xmm0,xmm0; movups [rcx+8],xmm0; movups [rcx+18h],xmm0; mov r14,[r9+10h]; cmp qword ptr [r9+18h],10h; mov [rbp+..],r9; jb`. Recipe 0 reads the displacement of `mov [rcx],edx`, which the decoder reports as 0 (no displacement bytes: the type is at the start of the object, right before mMessage at +8). There is one recipe: no second function stores the type at a place that is stable across builds, so the check is a window around the value, as for a field with one recipe. Every register, displacement and immediate byte is a wildcard. Evidence: the same constructor zeroes mMessage (a std::string at +8h) with the 16-byte stores at +8h and +18h on both builds, so mType is the member before it (the header member order)."

  recipe {
    step "pattern" {
      bytes  = "4D 89 ?? 4C 89 ?? ?? 48 89 ?? 89 ?? 0F 57 ?? 0F 11 ?? ?? 0F 11 ?? ?? 4D 8B ?? ?? 49 83 ?? ?? ?? 4C 89 ?? ?? 72 ??"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 32
  }
}

entry "GuiMessage.mUsername" {
  kind     = "field"
  notes    = "Offset in GuiMessage of mUsername, a std::string (32 bytes): 70h on 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (70.26), between mTTSMessage (50h) and mFullString (90h). Recipe 0 is the zeroing of the string in the constructor (`xorps xmm0,xmm0; movups [rsi+..],xmm0; movups [rsi+..],xmm0`; the second store is the string head) followed by the read of the username argument (`mov r15,[rbp+..]; mov r14,[r15+10h]; cmp qword ptr [r15+18h],10h; jb`), read as the displacement of the second store. Recipe 1 is the small-string test of the same copy: `mov r14,[r15+10h]; cmp qword ptr [r15+18h],10h; jb; mov rax,[rbp+..]; mov r15,[rax]; test r14,r14; js; lea rax,[rsi+..]`, read as the displacement of the lea. The two recipes are two blocks of the constructor and share only the argument read of the first lines. Every register, displacement, immediate and branch byte is a wildcard. The check is a window around the value."

  recipe {
    step "pattern" {
      bytes  = "0F 57 ?? 0F 11 ?? ?? ?? ?? ?? 0F 11 ?? ?? 4C 8B ?? ?? 4D 8B ?? ?? 49 83 ?? ?? ?? 72 ??"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4D 8B ?? ?? 49 83 ?? ?? ?? 72 ?? 48 8B 45 ?? 4C 8B ?? ?? 85 ?? 0F 88 ?? ?? ?? ?? 48 8D ?? ?? ?? ?? ??"
      offset = 27
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 64
    max = 192
  }
}

entry "GuiMessage.mFullString" {
  kind     = "field"
  notes    = "Offset in GuiMessage of mFullString, a std::string (32 bytes; `<username> message`, or the message alone when the username is empty): 90h on 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (70.26). Recipe 0 is the zeroing of the member in the constructor: `mov r14,[rbp+..]; xorps xmm0,xmm0; movups [rsi+..],xmm0; mov qword ptr [rsi+..],0; mov qword ptr [rsi+..],0Fh; mov byte ptr [rsi+..],0; movups [rsi+..],xmm0; movups [rsi+..],xmm0`, read as the displacement of the first store. Recipe 1 is the set-up of the three string pointers that follow (`lea rax,[rsi+..]; mov [rbp+..],rax; lea rax,[rsi+..]; mov [rbp+..],rax; test r15,r15; js; lea rax,[rsi+..]`), read as the displacement of the first lea. The two recipes are two blocks of the constructor and share no pattern bytes. Every register, displacement, immediate and branch byte is a wildcard. The check is a window around the value."

  recipe {
    step "pattern" {
      bytes  = "4C 8B ?? ?? ?? ?? ?? 0F 57 ?? 0F 11 ?? ?? ?? ?? ?? 48 C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? C6 ?? ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ??"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D ?? ?? ?? ?? ?? 48 89 45 ?? 48 8D ?? ?? ?? ?? ?? 48 89 45 ?? ?? 85 ?? 0F 88 ?? ?? ?? ?? 48 8D ?? ?? ?? ?? ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 96
    max = 224
  }
}

entry "GuiMessage.mFilteredFullString" {
  kind     = "field"
  notes    = "Offset in GuiMessage of mFilteredFullString, a std::optional<std::string> (32 bytes of string, then the engaged byte at +20h; see mFilteredFullStringEngaged): B0h on 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (70.26). Recipe 0 is the set-up of the three string pointers in the constructor (`lea rax,[rsi+..]; mov [rbp+..],rax; lea rax,[rsi+..]; mov [rbp+..],rax; test r15,r15; js; lea rax,[rsi+..]`), read as the displacement of the second lea. Recipe 1 is the copy of the filtered optional in the constructor: `cmp byte ptr [rdx+..],1; jne; lea rcx,[rsi+..]; call; nop; jmp; cmp byte ptr [rsi+..],1; jne; mov rax,[rsi+..]; cmp rax,10h`, read as the displacement of the lea. The two recipes are two blocks of the constructor and share no pattern bytes. Every register, displacement, immediate and branch byte is a wildcard. The check is a window around the value."

  recipe {
    step "pattern" {
      bytes  = "48 8D ?? ?? ?? ?? ?? 48 89 45 ?? 48 8D ?? ?? ?? ?? ?? 48 89 45 ?? ?? 85 ?? 0F 88 ?? ?? ?? ?? 48 8D ?? ?? ?? ?? ??"
      offset = 11
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "80 ?? ?? ?? 75 ?? 48 8D ?? ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 E9 ?? ?? ?? ?? 80 ?? ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 48 83 ?? ??"
      offset = 6
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 256
  }
}

entry "GuiMessage.mFilteredFullStringEngaged" {
  kind     = "field"
  notes    = "Offset in GuiMessage of the engaged byte of mFilteredFullString (the std::optional<std::string> flag, 1 when the filtered text is present): D0h on 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (70.26), 20h after mFilteredFullString (B0h). Recipe 0 is the zeroing in the constructor: `mov r14,[rbp+..]; xorps xmm0,xmm0; movups [rsi+..],xmm0; mov qword ptr [rsi+..],0; mov qword ptr [rsi+..],0Fh; mov byte ptr [rsi+..],0; movups [rsi+..],xmm0; movups [rsi+..],xmm0`, read as the displacement of the byte store. Recipe 1 is the copy of the filtered optional: `cmp byte ptr [rdx+..],1; jne; lea rcx,[rsi+..]; call; nop; jmp; cmp byte ptr [rsi+..],1; jne; mov rax,[rsi+..]; cmp rax,10h`, read as the displacement of the compare of the engaged byte. The two recipes are two blocks of the constructor and share no pattern bytes. Every register, displacement, immediate and branch byte is a wildcard. The check is a window around the value."

  recipe {
    step "pattern" {
      bytes  = "4C 8B ?? ?? ?? ?? ?? 0F 57 ?? 0F 11 ?? ?? ?? ?? ?? 48 C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? C6 ?? ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ??"
      offset = 39
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "80 ?? ?? ?? 75 ?? 48 8D ?? ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 E9 ?? ?? ?? ?? 80 ?? ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 48 83 ?? ??"
      offset = 24
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 320
  }
}

entry "GuiMessage.mAuthorXuid" {
  kind     = "field"
  notes    = "Offset in GuiMessage of mAuthorXuid, a std::string (32 bytes): D8h on 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (70.26), right after the engaged byte of mFilteredFullString (D0h). Recipe 0 is the zeroing in the constructor (the store block of mFullString): the last two 16-byte stores of `movups [rsi+..],xmm0; mov qword ptr [rsi+..],0; mov qword ptr [rsi+..],0Fh; mov byte ptr [rsi+..],0; movups [rsi+..],xmm0; movups [rsi+..],xmm0`, read as the displacement of the first of the two. Recipe 1 is the set-up of the three string pointers (`lea rax,[rsi+..]; mov [rbp+..],rax; lea rax,[rsi+..]; mov [rbp+..],rax; test r15,r15; js; lea rax,[rsi+..]`), read as the displacement of the third lea. The two recipes are two blocks of the constructor and share no pattern bytes. Every register, displacement, immediate and branch byte is a wildcard. The check is a window around the value."

  recipe {
    step "pattern" {
      bytes  = "4C 8B ?? ?? ?? ?? ?? 0F 57 ?? 0F 11 ?? ?? ?? ?? ?? 48 C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? C6 ?? ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ??"
      offset = 46
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D ?? ?? ?? ?? ?? 48 89 45 ?? 48 8D ?? ?? ?? ?? ?? 48 89 45 ?? ?? 85 ?? 0F 88 ?? ?? ?? ?? 48 8D ?? ?? ?? ?? ??"
      offset = 31
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 320
  }
}
