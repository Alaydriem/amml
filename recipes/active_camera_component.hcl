entry "ActiveCameraComponent.typeId" {
  kind     = "field"
  notes    = "The entt type id of MinecraftCamera::ActiveCameraComponent (the FNV-1a hash of the type name; an empty tag component, so only its sparse set is read): 113040A1h in 26.51. Recipe 0 is a view constructor over a storage with type id 8BCE84BEh and the ActiveCameraComponent storage (0x672b050: `lea rcx,[r14+30h]; mov edx,8BCE84BEh; call assure; mov rbx,rax; mov rdi,[rdi]; lea rcx,[rdi+30h]; mov edx,113040A1h; call`), keeping the other type id; recipe 1 is a view constructor over the CameraComponent, ActiveCameraComponent and a third storage (0x48bf8a0: `mov edx,4F6047C7h; call; mov rdi,rax; mov rcx,r14; mov edx,113040A1h; call; mov rbx,rax; mov rcx,r14; mov edx,..`), keeping the CameraComponent type id. Both read the immediate of the ActiveCameraComponent `mov edx`; both patterns cross one call (the assure call between the two type ids), whose bytes are wildcards. Every other displacement, immediate and branch byte is a wildcard. The former recipe 0 (a view constructor that assures another storage first, with that type id kept as a literal) is removed: 26.60 inlines that constructor, and every other site that assures the ActiveCameraComponent storage is an entt view constructor shared by several component sets, so no pattern is unique without keeping the value it reads. The remaining recipe keeps the CameraComponent type id as its literal and reads the second assure; it matches once on both builds. The value is the FNV-1a hash of the type name and does not change while the name stays."

  recipe {
    step "pattern" {
      bytes  = "48 89 D6 49 89 CE BA C7 47 60 4F E8 ?? ?? ?? ?? 48 89 C7 4C 89 F1 BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 C3 4C 89 F1 BA"
      offset = 22
      literal_ok = ["7: the CameraComponent type id in `mov edx,4F6047C7h`: the view constructors over other component pairs have the same code", "8: the CameraComponent type id in `mov edx,4F6047C7h`: the view constructors over other component pairs have the same code", "9: the CameraComponent type id in `mov edx,4F6047C7h`: the view constructors over other component pairs have the same code", "10: the CameraComponent type id in `mov edx,4F6047C7h`: the view constructors over other component pairs have the same code"]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 1
    max = 4294967295
  }
}
