entry "PacketSender.send" {
  kind     = "vslot"
  notes    = "Slot of void PacketSender::send(Packet&) in the PacketSender vtable: 2 on 26.51 (LeviLamina header order: dtor 0, isInitialized 1, send 2). Every caller of ClientInstance::getPacketSender then does `mov rcx,[rax]; mov r8,[rcx+10h]; lea rdx,[packet]; mov rcx,rax; mov rax,r8; call [guard]`. Recipe 0 is the ClientInstance function that opens the inventory (the InteractPacket with action 6 and serialization mode 5, as in ClientInstance.getPacketSender recipe 0), from the packet stores through the getPacketSender call to the send. Recipe 1 is another client function: `mov [rsp+..],eax; mov dword ptr [rsp+..],..; mov rax,[rsi]; mov rax,[rax+..]; mov rcx,rsi; call [guard]; mov rcx,[rax]; mov r8,[rcx+disp]; lea rdx,[rsp+..]; mov rcx,rax; mov rax,r8; call [guard]`. Both read the displacement of `mov r8,[rcx+disp]` as a slot; both patterns run past the getPacketSender call, because the send sequence alone is the same in some 40 callers. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 0..10 (the class has 12 virtuals). Recipe 0 is now a pattern_any that runs on past the send to tell the two identical send blocks of the 26.60 open-inventory function apart (the same alternatives as ClientInstance.getPacketSender recipe 0). Value 2 on both builds. What changed in 6ad79ee4-17d51891a1119914 (26.60): the function sends twice; recipe 1 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "C6 45 ?? ?? C7 45 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? 90 48 81 C4",
        "C6 45 ?? ?? C7 45 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 C7 E8 ?? ?? ?? ?? 48 81 FF ?? ?? ?? ?? 0F 84",
      ]
      offset = 33
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "89 44 24 ?? C7 44 24 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 41 ?? 48 8D 54 24 ?? 48 89 C1 4C 89 C0 FF 15"
      offset = 34
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 0
    max = 10
  }
}
