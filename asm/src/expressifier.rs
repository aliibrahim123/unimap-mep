use std::fmt::Write;

macro_rules! main_loop {
	($buf:expr, $label:lifetime => $inner:expr) => {
		$label: for p0 in 0..16 {
			write!($buf, "{p0} = {{ ").unwrap();
			for p1 in 0..16 {
				write!($buf, "{p1} = {{ ").unwrap();
				for p2 in 0..16 {
					write!($buf, "{p2} = {{ ").unwrap();
					for p3 in 0..16 {
						write!($buf, "{p3} = {{ ").unwrap();
						for p4 in 0..16 {
							write!($buf, "{p4} = {{ ").unwrap();
							for p5 in 0..16 {
								write!($buf, "{p5} = {{\n").unwrap();
								$inner;
								$buf.push_str("}, ");
							}
							$buf.push_str("}, ");
						}
						$buf.push_str("}, ");
					}
					$buf.push_str("}, ");
				}
				$buf.push_str("}, ");
			}
			$buf.push_str("}, ");
		}
	};
}

pub fn expressify(bin: &[u8]) -> String {
	let mut buf = String::new();
	buf.push_str("import cpu.entry { init_state, run_iter };\n");
	buf.push_str("import cpu.common { mem };\n");

	buf.push_str("fn init () => { ..init_state, mem = {");
	let mut lines = bin.chunks(16).peekable();
	main_loop!(buf, 'main => {
		for line_ind in 0..16 {
			write!(buf, "{line_ind} = [").unwrap();
			for byte in lines.next().unwrap() {
				write!(buf, "{}, ", byte & 0b1111).unwrap();
				write!(buf, "{}, ", byte >> 4).unwrap();
			}
			buf.push_str("],\n");
			if lines.peek().is_none() {
				buf.push_str("}}}}}}}");
				break 'main;
			}
		}
	});

	buf.push_str("};\n");

	buf.push_str("fn loop (state) => run_iter(state);\n");
	buf
}
