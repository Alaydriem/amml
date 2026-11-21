entry "FontRepository.mLoadedFonts" {
  kind     = "field"
  notes    = "Offset in FontRepository of mLoadedFonts, a std::vector<std::shared_ptr<Font>> (16-byte elements; a FontHandle's font id is an index into it): 40h on 26.63 and 70.26, the begin pointer; the end pointer is at +48h. The vector follows the bases (AppPlatformListener, EnableNonOwnerReferences) and mIsInitialized. It is read by the repository lookup of a font by id: `cmp r8,[rip+..]` (the id against the invalid-id sentinel) `; je; mov rax,[<repo>+48h]; sub rax,[<repo>+40h]; sar rax,4; cmp rax,r8; jae`, so the element count is the byte distance shifted by 4. Recipe 0 reads the displacement of the `sub` (the begin pointer). There is one recipe: the other reader of the vector on 26.63 is the same function, and FontHandle::getFont is a standalone function only on 70.26 (on 26.63 the compiler inlines it), so no second pattern fits both builds. The check is a window around the value. Every register, displacement and branch byte is a wildcard. Value 40h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). Evidence: on 70.26 the vector count `mov rcx,[<handle>+28h]; mov rdx,[rcx+48h]; sub rdx,[rcx+40h]; sar rdx,4; cmp rax,rdx; jae` in FontHandle::getFont (the function that loads the text `Font &FontHandle::getFont() const`) reads the same offsets from the repository that the FontHandle holds; the FontHandle mirror reads mFontRepository at 18h of the handle (FontHandle.size, 50h)."

  recipe {
    step "pattern" {
      bytes  = "4C 3B 05 ?? ?? ?? ?? 74 ?? 48 8B ?? ?? 48 2B ?? ?? 48 C1 F8 ?? 4C 39 C0"
      offset = 13
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
