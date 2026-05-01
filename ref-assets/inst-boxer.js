// gen by ai and refined by me
import * as fs from "fs";
import * as path from "path";
import insts from "./inst-def.js";
function generateDiagrams (instructions) {
	const svgWidth = 700;
	const bitWidth = svgWidth / 32;
	const boxHeight = 40;
	const subBoxHeight = 24;
	const svgHeight = 110;

	const outputDir = import.meta.dirname;
	instructions.forEach(inst => {
		let currentBit = 0;

		const parsedFields = inst.fields.map(field => {
			let width = 0, content = "", type = "", isFixed = false;
			if (typeof field === "number") {
				content = field.toString(2);
				width = content.length;
				isFixed = true;
			}
			else if (typeof field === "string") {
				content = field;
				width = field.length;
				isFixed = true;
			}
			else if (typeof field === "object") {
				content = field.name !== undefined ? field.name : (field.value !== undefined ? field.value.toString(2) : "");
				type = field.type || "";
				width = field.width || field.bits || content.length || 5;
				isFixed = field.value !== undefined || /^[01]+$/.test(content);
			}
			const startBit = currentBit;
			const endBit = currentBit + width - 1;
			currentBit += width;
			return { content, type, width, startBit, endBit, isFixed };
		});
		if (currentBit !== 32) {
			console.warn(`Warning: Instruction "${inst.name}" has ${currentBit} bits mapped instead of 32.`);
		}

		let svg = `<svg width="100%" height="${svgHeight}" viewBox="-20 0 ${svgWidth + 40} ${svgHeight}" xmlns="http://www.w3.org/2000/svg">\n`;

		svg += `<style>
      /* Light Mode Styles */
      .bit-idx { font-family: sans-serif; font-size: 13px; fill: #666; }
      .content-fixed { font-family: monospace; font-size: 16px; font-weight: bold; text-anchor: middle; fill: #111; }
      .content-var { font-family: sans-serif; font-size: 16px; text-anchor: middle; fill: #111; }
      .field-type { font-family: monospace; font-size: 13px; text-anchor: middle; fill: #555; }
      
      .box-line { stroke: #222; stroke-width: 1.5; fill: #fafafa; }
      /* Second row background (slightly shaded) */
      .box-type { stroke: #222; stroke-width: 1.5; fill: #eaecef; } 
      
      /* Dark Mode Overrides */
      @media (prefers-color-scheme: dark) {
        .bit-idx { fill: #a0a0a0; }
        .content-fixed { fill: #e0e0e0; }
        .content-var { fill: #e0e0e0; }
        .field-type { fill: #a0a0a0; }
        
        .box-line { stroke: #a0a0a0; fill: #1e1e1e; }
        .box-type { stroke: #a0a0a0; fill: #2d3136; }
      }
    </style>\n`;
		svg += `  <rect x="0" y="${25 + boxHeight}" width="${32 * bitWidth}" height="${subBoxHeight}" class="box-type" />\n`;

		parsedFields.forEach(f => {
			const x = (31 - f.endBit) * bitWidth;
			const w = f.width * bitWidth;

			svg += `  <rect x="${x}" y="25" width="${w}" height="${boxHeight}" class="box-line" />\n`;


			if (f.width === 1) {
				svg += `  <text x="${x + w / 2}" y="15" text-anchor="middle" class="bit-idx">${f.startBit}</text>\n`;
			} else {
				svg += `  <text x="${x + 4}" y="15" text-anchor="start" class="bit-idx">${f.endBit}</text>\n`;
				svg += `  <text x="${x + w - 4}" y="15" text-anchor="end" class="bit-idx">${f.startBit}</text>\n`;
			}

			if (f.isFixed) {
				let bits = f.content.padStart(f.width, '0');
				for (let i = 0; i < f.width; i++) {
					let bitCenter = x + (i * bitWidth) + (bitWidth / 2);
					svg += `  <text x="${bitCenter}" y="${25 + boxHeight / 2 + 5}" class="content-fixed">${bits[i]}</text>\n`;
				}
			} else {
				svg += `  <text x="${x + w / 2}" y="${25 + boxHeight / 2 + 5}" class="content-var">${f.content}</text>\n`;
			}

			if (f.type) {
				svg += `  <rect x="${x}" y="${25 + boxHeight}" width="${w}" height="${subBoxHeight}" class="box-type" />\n`;
				svg += `  <text x="${x + w / 2}" y="${25 + boxHeight + (subBoxHeight / 2) + 4}" class="field-type">${f.type}</text>\n`;
			}
		});
		svg += `</svg>`;

		const fileName = path.join(outputDir, `${inst.name}.svg`);
		fs.writeFileSync(fileName, svg);
	});
}
generateDiagrams(insts);