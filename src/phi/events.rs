
macro_rules! struct_events {
	( keyboard: { $( $k_alias:ident : $k_sdl:ident ),* }, other: { $( $o_alias:ident : $o_sdl:pat ),* } ) => {

		use sdl2::EventPump;


		pub struct ImmediateEvents {
			// For every keyboard event, we have an Option<bool>
			// Some(true)  => Was just pressed
			// Some(false) => Was just released
			// None        => Nothing happening _now_
			$( pub $k_alias: Option<bool> , )*
			$( pub $o_alias : bool , )*

			resize: Option<(u32, u32)>
		}

		impl ImmediateEvents {
			pub fn new() -> ImmediateEvents {
				ImmediateEvents {
					// When reinitialized, nothing has yet happened, so all are
					// set to None
					$( $k_alias: None , )*
					$( $o_alias: false , )*

					resize: None
				}
			}
		}


		pub struct Events {
			pump: EventPump,

			pub now: ImmediateEvents,

			// true  => pressed
			// false => not pressed
			$( pub $k_alias: bool ),*
		}

		impl Events {
			pub fn new(pump: EventPump) -> Events {
				Events {
					pump: pump,

					now: ImmediateEvents::new(),

					// By default, initialize every key with _not pressed_
					$( $k_alias: false ),*
				}
			}

			/// Update the events record.
			pub fn pump(&mut self, renderer: &mut ::sdl2::render::Renderer) {
				self.now = ImmediateEvents::new();

				for event in self.pump.poll_iter() {
					use sdl2::event::Event::*;
					use sdl2::keyboard::Keycode::*;

					match event {
						Window { win_event_id: ::sdl2::event::WindowEventId::Resized, .. } => {
							self.now.resize = Some(renderer.output_size().unwrap());
						},
						KeyDown { keycode, .. } => match keycode {
							// $( ... ),* containing $k_sdl and $k_alias means:
							//   "for every element ($k_alias : $k_sdl) pair,
							//    check whether the keycode is Some($k_sdl). If
							//    it is, then set the $k_alias fields to true."
							$(
								Some($k_sdl) => {
									// Prevent multiple presses when keeping a key down
									// Was previously not pressed?
									if !self.$k_alias {
										// Key pressed
										self.now.$k_alias = Some(true);
									}
									self.$k_alias = true;
								}
								),* // and add a comma after every option
							_ => {}
						},

						KeyUp { keycode, .. } => match keycode {
							$(
								Some($k_sdl) => {
									// Key released
									self.now.$k_alias = Some(false);
									self.$k_alias = false;
								}
								),*
							_ => {}
						},

						$(
							$o_sdl => {
								self.now.$o_alias = true;
							}
							),*

						_ => {
							$(
								self.now.$o_alias = false;
								),*
						}
					}
				}
			}
		}
	}
}

