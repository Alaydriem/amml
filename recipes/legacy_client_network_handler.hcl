entry "LegacyClientNetworkHandler.vtable" {
  kind     = "global"
  notes    = "The primary vtable of LegacyClientNetworkHandler (the live client handler, a ClientNetworkHandler and so a NetEventCallback). The class has no RTTI. Found from the strings `virtual void LegacyClientNetworkHandler::handle(const NetworkIdentifier &, const NetworkSettingsPacket &)` and `... const StartGamePacket &)`: each is loaded by one function, and each function sits in one vtable, at slot 87 and slot 235 (235 is the StartGamePacket handler slot that `inspect packets` gives). The same table holds the ContainerOpen and ContainerClose overrides at 193 and 192, where the ClientNetworkHandler.vtable holds the empty default. The table is stored by the constructor (that also stores the ClientNetworkHandler.vtable) and by the destructor; both also store a second base vtable at +240h. Recipe 0 is the constructor: `lea rax,[rip+..]; mov [r12],rax; lea rax,[rip+..]; mov [r12+240h],rax; xorps xmm0,xmm0; movups [r12+248h],xmm0`; it reads the RIP target of the first lea. Recipe 1 is the destructor: `mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; lea rax,[rip+..]; mov [rcx+240h],rax; mov rax,[rcx+258h]; test rax,rax; je; cmp byte ptr [rax],1`; it reads the RIP target of the first lea. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and RIP offset is a wildcard. Recipe 0 now runs on to the allocation after the stores (`mov rcx,[rip+..]; mov rax,[rcx]; mov rax,[rax+8]; mov edx,imm32; call [guard]`), because a screen controller constructor has the same store run on 26.60. The second base is at +240h on 26.51 and +298h on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): the class grew before the second base (+58h), and recipe 0 had 2 matches; recipe 1 (the destructor) agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 49 89 04 24 48 8D 05 ?? ?? ?? ?? 49 89 84 24 ?? ?? ?? ?? 0F 57 C0 41 0F 11 84 24 ?? ?? ?? ?? 48 8B 0D ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? BA ?? ?? ?? ?? FF 15" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 05 ?? ?? ?? ?? 48 89 81 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 85 C0 74 ?? 80 38 ??"
      offset = 3
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
