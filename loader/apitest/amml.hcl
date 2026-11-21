namespace   = "amml"
name        = "apitest"
entry       = "amml_apitest.dll"
api_version = 3
uses {
  feature "core" {
    event "mods.loaded" {
      version  = 1
      required = true
    }
    event "registry.provider_ready" {
      version  = 1
      required = true
    }
    event "frame.update" {
      version  = 1
      required = true
    }
    event "client.tick" {
      version  = 1
      required = true
    }
    event "loader.shutdown" {
      version  = 1
      required = false
    }
  }
  feature "world" {
    event "world.joined" {
      version  = 1
      required = false
    }
    event "world.left" {
      version  = 1
      required = false
    }
  }
  feature "client" {
    area "client.read" {
      version  = 1
      required = false
    }
    area "client.top_screen" {
      version  = 2
      required = false
    }
    area "client.hud" {
      version  = 1
      required = false
    }
    area "client.mouse" {
      version  = 1
      required = false
    }
    area "client.pause" {
      version  = 1
      required = false
    }
  }
  feature "input" {
    event "input.key" {
      version  = 1
      required = false
    }
    event "input.mouse" {
      version  = 1
      required = false
    }
    event "input.look_delta" {
      version  = 1
      required = false
    }
    event "input.player_move" {
      version  = 1
      required = false
    }
    event "input.gamepad_poll" {
      version  = 1
      required = false
    }
    event "input.injected_released" {
      version  = 2
      required = false
    }
    area "input.lease" {
      version  = 1
      required = false
    }
    area "input.key_inject" {
      version  = 1
      required = false
    }
    area "input.mouse_inject" {
      version  = 1
      required = false
    }
    area "input.turn" {
      version  = 1
      required = false
    }
    area "input.move" {
      version  = 1
      required = false
    }
  }
  feature "window" {
    event "window.pointer" {
      version  = 1
      required = false
    }
    event "window.char" {
      version  = 1
      required = false
    }
    event "window.size" {
      version  = 1
      required = false
    }
    event "window.focus" {
      version  = 1
      required = false
    }
    area "window.read" {
      version  = 1
      required = false
    }
    area "window.request_size" {
      version  = 1
      required = false
    }
  }
  feature "overlay" {
    event "overlay.frame_present" {
      version  = 1
      required = false
    }
    event "overlay.frame_presented" {
      version  = 1
      required = false
    }
    event "overlay.swap_chain_resized" {
      version  = 1
      required = false
    }
    event "overlay.surface_lost" {
      version  = 1
      required = false
    }
    event "overlay.render_device_reset" {
      version  = 1
      required = false
    }
    event "overlay.screenshot_taken" {
      version  = 1
      required = false
    }
    event "overlay.screenshot_written" {
      version  = 1
      required = false
    }
    area "overlay.surface" {
      version  = 1
      required = false
    }
    area "overlay.screenshot" {
      version  = 1
      required = false
    }
    area "overlay.capture" {
      version  = 1
      required = false
    }
  }
}
