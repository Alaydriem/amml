entry "MinecraftInputHandler.mInputHandler" {
  kind  = "field"
  notes = "Offset of `mInputHandler` (std::unique_ptr<InputHandler>, 8 bytes) in MinecraftInputHandler: 20h in 26.63 (6aed1906-b127418e74d2f2f9) and in 70.26 (6afa4e47-dadc375797a3d31a). The MinecraftInputHandler is the object that ClientInstance.getMinecraftInput hands out in its NonOwnerPointer (the pointer at +10h of the 24-byte result). The loader reads it for Client::note_activity: getMinecraftInput()->mInputHandler->mAnyInputHandler(client). How it was found: MinecraftInputHandler::init (named by the RTTI of its lambda `init@MinecraftInputHandler` on 26.63) allocates the InputHandler (310h bytes on 26.63, 300h on 70.26), calls the InputHandler constructor, then resets the unique_ptr: `lea r12,[rbp+..]; mov r14,[rsi+20h]; mov rax,[rbp+..]; mov [rsi+20h],rax; test r14,r14; je; mov rcx,r14; call` (the InputHandler destructor, then the sized delete). Recipe 0 is that reset, a pattern_any with one alternative per build: alternative 0 the 26.63 registers (r12, r14) and a disp8 frame load, alternative 1 the 70.26 registers (r14, r15) and a disp32 frame load; each matches once on its own build and not on the other. Offset 4 is the load of the old pointer, whose displacement is the member. It ends at the destructor call. Every displacement and branch byte is a wildcard. Second evidence on both builds (not a recipe, no unique anchor): MinecraftInputHandler::_registerInputHandlers loads `mov rcx,[rsi+20h]` before each InputHandler::register*Handler call (the any-input handler among them, see InputHandler.mAnyInputHandler). The range check is a window around the value. One recipe and a range check, as for other members."

  recipe {
    step "pattern_any" {
      patterns = [
        "4C 8D 65 ?? 4C 8B 76 ?? 48 8B 45 ?? 48 89 46 ?? 4D 85 F6 74 ?? 4C 89 F1 E8",
        "4C 8D 75 ?? 4C 8B 7E ?? 48 8B 85 ?? ?? ?? ?? 48 89 46 ?? 4D 85 FF 74 ?? 4C 89 F9 E8",
      ]
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 64
  }
}
