entry "GuiMessage.ctor" {
  kind     = "function"
  hooked   = true
  notes    = "GuiMessage::GuiMessage(MessageType type, std::string const& username, std::string const& message, std::optional<std::string> filteredMessage, std::string const& ttsMessage, std::string const& authorXuid, float lifetime, bool forceVisible, bool ttsRequired): rcx = this, edx = type, r8 = username, r9 = message, [rsp+28h] = a pointer to the filteredMessage temporary (passed by value: the callee destroys it, so a hook must not read it after the original returns), [rsp+30h] = ttsMessage, [rsp+38h] = authorXuid, [rsp+40h] = lifetime, [rsp+48h] = forceVisible, [rsp+50h] = ttsRequired; it returns this in rax. It copies the arguments to mType +0h, mMessage +8h, mFilteredMessage +28h (optional, engaged byte +48h), mTTSMessage +50h, mUsername +70h, mAuthorXuid +D8h, forceVisible +F8h, ttsRequired +F9h, lifetime +FCh, then builds mFullString +90h (`<username> message`, or the message alone when the username is empty) and mFilteredFullString +B0h (optional, engaged byte +D0h). sizeof(GuiMessage) is 110h. Found from the TextPacket handler (slot 244 of ClientNetworkHandler.vtable): each GuiData display function calls GuiData::_addMessage (0xe8e420 on 26.60), which calls this constructor once (0xe8e9d5); the only other caller (0xe8cea0) makes type 9 messages for another list. Recipe 0 is the body after the last argument copy (`movss xmm0,[rbp+..]; movzx eax,byte ptr [rbp+..]; movzx ecx,byte ptr [rbp+..]; mov [rsi+..],cl; mov [rsi+..],al; movss [rsi+..],xmm0; mov word ptr [rsi+..],0; mov dword ptr [rsi+..],0; mov byte ptr [rsi+..],0`), then function_start_of. Recipe 1 is the call in GuiData::_addMessage: the stack argument stores and the register setup (`lea r8,[r9+..]; mov edx,[r9]; mov [rsp+..],al; mov [rsp+..],cl; movss [rsp+..],xmm0; mov rax,[rbp+..]; mov [rsp+..],rax; lea rax,[rbp+..]; mov [rsp+..],rax; lea rax,[rbp+..]; mov [rsp+..],rax; lea rcx,[rbp+..]; lea r9,[rbp+..]; call`), read as a branch target. They are in two functions. Every displacement, immediate and branch byte is a wildcard. In 6ad79ee4-17d51891a1119914 (26.60) the constructor is at 0xE8D420. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes = "F3 0F 10 85 ?? ?? ?? ?? 0F B6 85 ?? ?? ?? ?? 0F B6 8D ?? ?? ?? ?? 88 8E ?? ?? ?? ?? 88 86 ?? ?? ?? ?? F3 0F 11 86 ?? ?? ?? ?? 66 C7 86 ?? ?? ?? ?? ?? ?? C7 86 ?? ?? ?? ?? ?? ?? ?? ?? C6 86 ?? ?? ?? ?? ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "4D 8D 41 ?? 41 8B 11 88 44 24 ?? 88 4C 24 ?? F3 0F 11 44 24 ?? 48 8B 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8"
      offset = 71
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
