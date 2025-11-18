import init, { Buffer } from "./pkg/mt_wasm.js";

/**
 * @typedef {import("./pkg/my_wasm.js").Buffer} Buffer
 * @typedef {import("./pkg/my_wasm.js")} initOutput
 */

let N = 0;
let T = 0;
let A_initial = [];
let B_target = [];
let operations = [];
/** @type {Buffer} */
let buffer = null;
/** @type {initOutput} */
let initOutput = null;
let parsedOutputOps = [];

const MAX_GRID_VALUE = 50;
const SVG_VIEW_SIZE = 400;

function valueToColor(value) {
	const hue = (1 - (value / MAX_GRID_VALUE)) * 240;
	return `hsl(${hue}, 80%, 50%)`;
}

function diffToColor(diff) {
	const maxDiff = MAX_GRID_VALUE;
	const lightness = 95 - (diff / maxDiff) * 60;
	return `hsl(0, 80%, ${lightness}%)`;
}

function drawGrid(elementId, grid, colorFunc) {
	const svg = document.getElementById(elementId);
	if (!svg || N === 0) return;
	const currentRenderSize = Math.min(SVG_VIEW_SIZE, 50 * N);
	svg.setAttribute('width', currentRenderSize);
	svg.setAttribute('height', currentRenderSize);
	while (svg.firstChild) {
		svg.removeChild(svg.firstChild);
	}

	const cellSize = currentRenderSize / N;

	for (let i = 0; i < N; i++) {
		for (let j = 0; j < N; j++) {
			const value = grid[i * N + j];
			const color = colorFunc(value);

			const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
			rect.setAttribute('x', j * cellSize);
			rect.setAttribute('y', i * cellSize);
			rect.setAttribute('width', cellSize);
			rect.setAttribute('height', cellSize);
			rect.setAttribute('fill', color);
			rect.setAttribute('stroke', '#333');
			rect.setAttribute('stroke-width', 0.5);
			const title = document.createElementNS('http://www.w3.org/2000/svg', 'title');
			title.textContent = `(${i}, ${j}): ${value}`;
			rect.appendChild(title);

			svg.appendChild(rect);
		}
	}

	if (buffer.get_time() > 0) {
		const { op_id, x, y } = parsedOutputOps[buffer.get_time() - 1];
		const data = operations[op_id];
		const { H, W } = data;
		const applyRange = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
		applyRange.setAttribute('x', y * cellSize);
		applyRange.setAttribute('y', x * cellSize);
		applyRange.setAttribute('width', W * cellSize);
		applyRange.setAttribute('height', H * cellSize);
		applyRange.setAttribute('fill', 'none');
		applyRange.setAttribute('stroke', 'white');
		applyRange.setAttribute('stroke-width', 2);
		svg.appendChild(applyRange);
	}
}

function display() {
	if (N === 0) return;

	let sumSquaredError = 0;
	const A_grid = [];
	const diffGrid = [];

	const mem = new Uint32Array(initOutput.memory.buffer);
	const now_state_ptr = buffer.now_state_ptr() / 4;
	const b_state_ptr = buffer.b_state_ptr() / 4;

	for (let i = 0; i < N; i++) {
		for (let j = 0; j < N; j++) {
			const diff = mem[now_state_ptr + i * N + j] - mem[b_state_ptr + i * N + j];
			sumSquaredError += diff * diff;
			A_grid[i * N + j] = mem[now_state_ptr + i * N + j];
			diffGrid[i * N + j] = Math.abs(diff);
		}
	}

	const score = 1e9 * (N * N) / ((N * N) + buffer.get_time() + sumSquaredError);

	document.getElementById('score-display').textContent = score.toFixed();
	document.getElementById('error-display').textContent = (sumSquaredError / (N * N)).toFixed(2);

	drawGrid('grid-A-prime', A_grid, valueToColor);
	drawGrid('grid-diff', diffGrid, diffToColor);
}

const loadInput = window.loadInput = function () {
	const inputData = document.getElementById('input-data-area').value;
	const lines = inputData.trim().split('\n').map(line => line.trim()).filter(line => line.length > 0);

	if (lines.length === 0) {
		alert("入力データが空です。");
		return;
	}

	const NT = lines.shift().split(/\s+/).map(Number);
	N = NT[0];
	T = NT[1];

	if (N < 15 || T < 1) {
		alert(`N=${N}, T=${T}。制約範囲外の可能性があります。`);
	}

	A_initial = [];
	for (let i = 0; i < N; i++) {
		A_initial.push(lines.shift().split(/\s+/).map(Number));
	}

	B_target = [];
	for (let i = 0; i < N; i++) {
		B_target.push(lines.shift().split(/\s+/).map(Number));
	}

	operations = [];
	const opDefsHTML = document.getElementById('op-definitions');
	opDefsHTML.innerHTML = '';
	for (let i = 0; i < T; i++) {
		const op = lines.shift().split(/\s+/).map(Number);
		const [H, W, a, b] = op;
		operations.push({ H, W, a, b, id: i });

		const opItem = document.createElement('div');
		opItem.classList.add('op-item');
		opItem.id = `op-def-${i}`;
		opItem.textContent = `ID ${i}: Replace(${H}x${W}, ${a} -> ${b})`;
		opDefsHTML.appendChild(opItem);
	}

	document.getElementById('max-uses').textContent = T * 3;
	buffer.set_first_state(A_initial.flat(), N);
	buffer.set_op(operations.map(op => [op.H, op.W, op.a, op.b]).flat());
	buffer.update_applications(parsedOutputOps.map(op => [op.op_id, op.x, op.y]).flat());
	display();
	const b_state_ptr = buffer.b_state_ptr() / 4;
	const mem = new Uint32Array(initOutput.memory.buffer);
	drawGrid('grid-B', mem.subarray(b_state_ptr, b_state_ptr + N * N), valueToColor);
}

function parseOutputData() {
	document.getElementById('operation-slider').max = 0;
	if (N === 0) {
		alert("まず入力データを読み込んでください。");
		return false;
	}

	const outputData = document.getElementById('output-data-area').value;
	const lines = outputData.trim().split('\n').map(line => line.trim()).filter(line => line.length > 0);

	if (lines.length === 0) {
		parsedOutputOps = [];
		document.getElementById('op-count').textContent = 0;
		return true;
	}

	const numOps = Number(lines.shift());

	if (isNaN(numOps) || numOps !== lines.length) {
		alert("出力形式が不正です。1行目に操作の総数を指定してください。");
		return false;
	}

	parsedOutputOps = [];
	for (let k = 0; k < numOps; k++) {
		const parts = lines[k].split(/\s+/).map(Number);
		if (parts.length !== 3) {
			alert(`操作 ${k} の形式が不正です: ${lines[k]}`);
			return false;
		}
		if (parts[0] < 0 || parts[0] >= T) {
			alert(`操作 ${k} の op_id が不正です: ${parts[0]}`);
			return false;
		}
		if (parts[1] < 0 || parts[1] + operations[parts[0]].H > N ||
			parts[2] < 0 || parts[2] + operations[parts[0]].W > N) {
			alert(`操作 ${k} の適用位置が不正です: (${parts[1]}, ${parts[2]})`);
			return false;
		}
		parsedOutputOps.push({ op_id: parts[0], x: parts[1], y: parts[2] });
	}
	document.getElementById('op-count').textContent = parsedOutputOps.length;
	document.getElementById('operation-slider').max = parsedOutputOps.length.toString();
	return true;
}

window.applyOutput = function () {
	if (!parseOutputData()) return;
	buffer.update_applications(parsedOutputOps.map(op => [op.op_id, op.x, op.y]).flat());
	display();
	updateCurrentOpDisplay(parsedOutputOps.length);
}

function updateCurrentOpDisplay(index) {
	document.getElementById('current-op-index').textContent = index;
	document.getElementById('operation-slider').value = index;
}

window.applySlider = function () {
	const targetIndex = Number(document.getElementById('operation-slider').value);
	if (targetIndex < 0 || targetIndex > parsedOutputOps.length) return;

	if (targetIndex > 0) {
		const opInfo = parsedOutputOps[targetIndex - 1];
		document.getElementById(`op-def-${opInfo.op_id}`).classList.add('current');
		document.getElementById('op-def-' + opInfo.op_id).scrollIntoView({ behavior: 'instant', block: 'center' });
	}
	if (!buffer.set_time(targetIndex)) {
		throw new Error("Failed to set time via slider");
	}
	display();
	updateCurrentOpDisplay(buffer.get_time());
}

const generateRandomData = window.generateRandomData = function () {
	const seed = Number(document.getElementById('seed').value);
	buffer.free();
	buffer = new Buffer();
	buffer.random_gen(seed);
	document.getElementById('input-data-area').value = buffer.to_string().trim();
	document.getElementById('output-data-area').value = '';
	loadInput();
}
document.addEventListener('DOMContentLoaded', async () => {
	initOutput = await init();
	buffer = new Buffer();
	generateRandomData();
});
