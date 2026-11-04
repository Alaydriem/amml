entry "RenderMaterialInfo.mMaterial" {
  kind     = "field"
  notes    = "Offset of the mce::RenderMaterial pointer in mce::RenderMaterialInfo: 40h on 6ad79ee4-17d51891a1119914 (26.60). Found in the draw-state builder 0x66b1f30 that the mesh insert calls once per mesh (call at 0x6649253; its first output is stored to [rbp+1E8h] there). It takes the output state in rcx and a pointer to the RenderMaterialInfo pointer in rdx, and reads `mov rcx,[rdx]; mov rdx,[rcx+40h]`; every legacy state it then converts is read through that rdx: enableBlend at +185h (`cmp byte ptr [rdx+185h],1`), the blend factors at +180h and +181h, the cull mode at +1A8h, depthTestEnabled at +188h and depthWriteMask at +193h (see RenderMaterial.mEnableBlend and RenderMaterial.mDepthWriteMask). Recipe 0 is `mov rcx,[rdx]; mov rdx,[rcx+..]; xor r11d,r11d; cmp byte ptr [rdx+..],imm; jne`; it reads the displacement of operand 1 of the second mov at offset 3. Every displacement, immediate and branch byte is a wildcard. Range window 20h..80h. The entry has one recipe and a range check (the rule for a field). 26.51 is not derived for this entry."

  recipe {
    step "pattern" {
      bytes  = "48 8B 0A 48 8B 51 ?? 45 31 DB 80 BA ?? ?? ?? ?? ?? 0F 85"
      offset = 3
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
