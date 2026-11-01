entry "ClientEntityWaypoint.vtable" {
  kind     = "global"
  notes    = "The primary vtable of ClientEntityWaypoint (a ClientWaypoint that follows an actor; the client makes one for every LocatorBarPacket Add whose payload has an mActorID). The class has no RTTI. 0xF13DFD0 on 26.60, 0xE7D08B0 on 26.51. Only two places refer to it (`inspect xrefs`): the waypoint factory in the ClientLocatorBar packet function (0x1e8a280 on 26.60, 0x1b82c40 on 26.51) and the destructor (0x2555230 on 26.60, 0x1b5d210 on 26.51, slot 0). Slot 7 is tryGetActorID (0x25149f0 on 26.60: `mov rax,rdx; mov rcx,[rcx+78h]; mov [rdx],rcx; mov byte ptr [rdx+8],1; ret`), slot 8 the update that follows the actor. Layout set by the factory: the ClientWaypoint part (98h bytes in all), the ActorUniqueID at +78h, an entity reference at +80h. Recipe 0 is the factory: after `operator new(98h)` it clears the ClientWaypoint members, then `mov byte ptr [rax+74h],imm; lea rcx,[rip+..]; mov [rax],rcx; mov rcx,[r12+..]; mov [rax+78h],rcx (the actor id); movups [rax+80h],xmmN; mov dword ptr [rax+90h],imm`; it reads the RIP target of the lea. The ModRM byte of the movups is a wildcard (xmm6 on 26.60, xmm7 on 26.51). The actor id store keeps it apart from the plain ClientWaypoint branch next to it. There is one recipe. No compliant independent second recipe exists: the only other reference is the destructor, and every pattern there that ends at a call is ambiguous. `mov edi,edx; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; mov rcx,[rcx+..]; ... call [guard]` matches 22 destructors (14 with the prologue). With the entity reference offset 88h kept it still matches 2 (the second is another class with the same layout). The inlined ClientWaypoint base part after the call matches 16. The deleting tail with sizeof 98h kept matches 14. The pattern that is unique joins the two parts across a `call [guard]`, which the recipe rules forbid. tryGetActorID (slot 7) is a leaf, and no step goes from a function to the vtable that holds it. Every displacement, immediate and branch byte is a wildcard. The section check keeps the value in .rdata."

  recipe {
    step "pattern" {
      bytes  = "C6 40 ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 08 49 8B 4C 24 ?? 48 89 48 ?? 0F 11 ?? ?? ?? ?? ?? C7 80 ?? ?? ?? ?? ?? ?? ?? ??"
      offset = 4
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
