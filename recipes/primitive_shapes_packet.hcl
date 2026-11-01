entry "PrimitiveShapesPacket.mShapes" {
  kind     = "field"
  notes    = "Offset of `mShapes` (std::vector<PrimitiveShapeDataPayload>) in PrimitiveShapesPacket: 30h in 26.51 (LeviLamina header: PayloadPacket<PrimitiveShapesPacketPayload>, the Packet base is 30h bytes and the payload holds only the vector; the packet is 50h bytes, make_shared allocates 60h). One recipe: the shape system that ClientNetworkHandler::handle(PrimitiveShapesPacket) calls (0x6686330) walks the vector of the packet in rdx (`mov rdi,[rdx+30h]; mov r15,[rdx+38h]; cmp rdi,r15; je`); it reads the displacement of the first load. The make_shared function of the packet (0x16bffd0) zeroes the vector at +40h of the allocation, past the 10h-byte control block, but its code is shared by six other packets. Grade: the header sum (Packet 30h). Every displacement and branch byte is a wildcard. What changed in 6ad79ee4-17d51891a1119914: the shape system keeps the vector end in r14 instead of r15, and another function walks a vector of another packet at +20h/+28h with the old pattern, so the old pattern matched twice; the pattern now wildcards the end register and adds the start of the loop setup (`mov rsi,rcx; lea rbx,[rbp-20h]; xorps xmm6,xmm6; jmp`), and the step pattern wildcards the end register. The value did not change (30h, B8h). Evidence on 26.60, from the 26.51 shape system: (1) a draft of its existing-shape update (`cmp byte ptr [rdi+9],1; jne; mov rax,[rcx]; mov rax,[rax+10h]; mov rdx,rdi; call`) matches once on 26.60, in the function that starts with the anchor; (2) the whole function lines up instruction for instruction with the 26.51 one (vector loads at 30h/38h, `add rdi,0B8h` step, the 20h-byte shape list walk)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 7A ?? 4C 8B ?? ?? 4C 39 ?? 0F 84 ?? ?? ?? ?? 48 89 CE 48 8D 5D ?? 0F 57 F6 EB"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 96
  }
}
