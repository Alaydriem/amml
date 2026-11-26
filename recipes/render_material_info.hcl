entry "RenderMaterialInfo.mMaterial" {
  kind     = "field"
  notes    = "Offset of the mce::RenderMaterial pointer in mce::RenderMaterialInfo: 40h on 6ad79ee4-17d51891a1119914 (26.60). Found in the draw-state builder 0x66b1f30 that the mesh insert calls once per mesh (call at 0x6649253; its first output is stored to [rbp+1E8h] there). It takes the output state in rcx and a pointer to the RenderMaterialInfo pointer in rdx, and reads `mov rcx,[rdx]; mov rdx,[rcx+40h]`; every legacy state it then converts is read through that rdx: enableBlend at +185h (`cmp byte ptr [rdx+185h],1`), the blend factors at +180h and +181h, the cull mode at +1A8h, depthTestEnabled at +188h and depthWriteMask at +193h (see RenderMaterial.mEnableBlend and RenderMaterial.mDepthWriteMask). Recipe 0 is `mov rcx,[rdx]; mov rdx,[rcx+..]; xor r11d,r11d; cmp byte ptr [rdx+..],imm; jne`; it reads the displacement of operand 1 of the second mov at offset 3. Every displacement, immediate and branch byte is a wildcard. Range window 20h..80h. The entry has one recipe and a range check (the rule for a field). 26.51 is not derived for this entry. The recipe is a pattern_any with one alternative for each build shape; alternative 0 has five leading wildcards so that one offset serves both. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the value is 48h (a member was added before it) and the builder tests the info pointer and the material pointer before it reads the blend state (`mov rax,[rdx]; test rax,rax; je; mov r14,[rax+48h]; test r14,r14; jne`), so alternative 1 reads the displacement of that load. Class: real code change (layout, the value moved by 8). Evidence on 1.26.70.26: (1) the builder is found from its header stores (`mov byte ptr [rcx+0Eh],2; mov dword ptr [rcx+10h],0; mov byte ptr [rcx+14h],0`), a pattern drafted from the 26.63 builder that matches once on each build, in a function with the same nine direct callers on both builds; (2) in that builder the pointer loaded at 48h is the one through which the blend state is read: RenderMaterial.mEnableBlend (185h), mBlendSource (180h) and mBlendDestination (181h), the entries that resolve on this build with their 26.63 values, as the load at 40h feeds them on 26.63."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? ?? ?? ?? ?? 48 8B 0A 48 8B 51 ?? 45 31 DB 80 BA ?? ?? ?? ?? ?? 0F 85",
        "48 8B 02 48 85 C0 74 ?? 4C 8B 70 ?? 4D 85 F6 75 ??",
      ]
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 128
  }
}
