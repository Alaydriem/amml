entry "SerializedSkinImpl.mSkinImage" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mSkinImage (SkinImage, an mce::Image) in the object that a SerializedSkinRef's shared_ptr element points to: 80h on 26.60, A0h on 26.51 (26.60 removed std::string mPlayFabId after mId). SerializedSkinImpl sits at +0 of Bedrock::Application::ThreadOwner<SerializedSkinImpl>: every read below is relative to the element pointer, and the offsets match the header layout counted from 0 (mDefaultGeometryName 60h, mGeometryData F8h on 26.60). mce::Image is the same on both builds: +0 format, +4 width, +8 height, +0Ch depth, +10h usage, +18h the mce::Blob (deleter +18h, data pointer +20h, byte size +28h). Found in the function that returns the byte size of a skin (0x298f4e0 on 26.60, 0x2650180 on 26.51; called by the skin memory check of the PlayerListPacket and PlayerSkinPacket handlers): it adds the cape image size, the skin image size and the size of each animated image (`mov rcx,[rcx]; mov rbx,[rcx+capeSize]; mov rax,[rcx+..]; add rbx,[rcx+skinSize]; mov rdx,[rcx+..]; cmp rax,rdx; je; xchg ax,ax; add rbx,[rax+30h]; add rax,40h; cmp rax,rdx; jne`). One recipe: that pattern from `mov rdi,rcx`, read as the displacement of `add rbx,[rcx+..]` (the skin image byte size: A8h on 26.60, C8h on 26.51), then `add -28h` (Image.mImageBytes 18h + Blob.mSize 10h) to reach the image start. Every displacement, immediate and branch byte is a wildcard, the two-byte nop included. The range check keeps the value plausible."

  recipe {
    step "pattern" {
      bytes  = "48 89 CF 48 8B 09 48 8B 99 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 03 99 ?? ?? ?? ?? 48 8B 91 ?? ?? ?? ?? 48 39 D0 74 ?? ?? ?? 48 03 58 ?? 48 83 C0 ?? 48 39 D0 75"
      offset = 20
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" { value = -40 }
  }

  check "range" {
    min = 96
    max = 192
  }
}

entry "SerializedSkinImpl.mGeometryData" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mGeometryData (Json::Value, 16 bytes; the skin geometry, with the head bone UV of persona and custom skins) in the object that a SerializedSkinRef's shared_ptr element points to: F8h on 26.60, 118h on 26.51 (26.60 removed std::string mPlayFabId after mId). It matches the header layout counted from 0: mSkinImage 80h, mCapeImage B0h, mSkinAnimatedImages E0h (24 bytes), then mGeometryData F8h on 26.60. The validity test of a SerializedSkinRef (0x6bdee80 on 26.60) also reads a Json::Value at +F8h. Found in the skin size function (0x298f4e0 on 26.60, 0x2650180 on 26.51): after the animated image loop it serialises the geometry with JsonValue.toStyledString (`cmp rax,rdx; jne; add rcx,imm32; lea rdx,[rsp+..]; call`) to count its bytes. One recipe: that pattern, read as the immediate of the add. The same bytes match once on both builds. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible."

  recipe {
    step "pattern" {
      bytes  = "48 39 D0 75 ?? 48 81 C1 ?? ?? ?? ?? 48 8D 54 24 ?? E8"
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 192
    max = 320
  }
}
