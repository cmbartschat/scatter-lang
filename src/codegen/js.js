/* eslint-disable quotes */
/* eslint-disable camelcase */
/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable no-empty */
/* eslint-disable no-constant-condition */
/* eslint-disable no-console */
const STATE = {
  values: [],
}

const STACK_UNDERFLOW = 101
const STACK_OVERFLOW = 102
const STRING_MAX = 103
const TYPE_MISMATCH = 104
const ASSERT_FAILED = 105

const NOT_IMPLEMENTED = 201
const DATA_CORRUPTED = 202
const STRING_TOO_LONG = 203

function assertStackHas(x) {
  if (STATE.values.length < x) {
    throw new Error(STACK_UNDERFLOW)
  }
}

function readStack(offset) {
  return STATE.values[STATE.values.length + offset]
}

function storeStack(offset, v) {
  STATE.values[STATE.values.length + offset] = v
}

function readStackNumber(offset) {
  const v = readStack(offset)
  if (typeof v !== 'number') {
    throw TYPE_MISMATCH
  }
  return v
}

function readStackString(offset) {
  const v = readStack(offset)
  if (typeof v !== 'string') {
    throw TYPE_MISMATCH
  }
  return v
}

function push(v) {
  STATE.values.push(v)
}

function swap() {
  assertStackHas(2)
  const temp = readStack(-2)
  storeStack(-2, readStack(-1))
  storeStack(-1, temp)
}

function drop() {
  STATE.values.pop()
}

function join() {
  assertStackHas(2)
  storeStack(-2, `${readStack(-2)}${readStack(-1)}`)
  drop()
}

function plus() {
  assertStackHas(2)
  storeStack(-2, readStackNumber(-2) + readStackNumber(-1))
  drop()
}

function minus() {
  assertStackHas(2)
  storeStack(-2, readStackNumber(-2) - readStackNumber(-1))
  drop()
}

function greater() {
  assertStackHas(2)
  storeStack(-2, readStackNumber(-2) > readStackNumber(-1))
  drop()
}

function less() {
  assertStackHas(2)
  storeStack(-2, readStackNumber(-2) < readStackNumber(-1))
  drop()
}
function times() {
  assertStackHas(2)
  storeStack(-2, readStackNumber(-2) * readStackNumber(-1))
  drop()
}
function divide() {
  assertStackHas(2)
  storeStack(-2, readStackNumber(-2) / readStackNumber(-1))
  drop()
}
function modulo() {
  assertStackHas(2)
  storeStack(-2, readStackNumber(-2) % readStackNumber(-1))
  drop()
}
function pow_i() {
  assertStackHas(2)
  storeStack(-2, readStackNumber(-2) ** readStackNumber(-1))
  drop()
}

function and_i() {
  assertStackHas(2)
  storeStack(-2, readStack(-2) && readStack(-1))
  drop()
}

function or_i() {
  assertStackHas(2)
  storeStack(-2, readStack(-2) || readStack(-1))
  drop()
}

function rot() {
  assertStackHas(3)
  const first = readStack(-3)
  storeStack(-3, readStack(-2))
  storeStack(-2, readStack(-1))
  storeStack(-1, first)
}

function increment() {
  assertStackHas(1)
  storeStack(-1, readStackNumber(-1) + 1)
}
function decrement() {
  assertStackHas(1)
  storeStack(-1, readStackNumber(-1) - 1)
}

function substring() {
  assertStackHas(3)
  const str = readStackString(-3)
  const start = readStackNumber(-2)
  const end = readStackNumber(-1)
  storeStack(-3, str.substring(start, Math.max(start, end)))
  drop()
  drop()
}

function length() {
  assertStackHas(1)
  storeStack(-1, readStackString(-1).length)
}

function over() {
  assertStackHas(2)
  storeStack(0, readStack(-2))
}

function equals() {
  assertStackHas(2)
  const left = readStack(-2)
  const right = readStack(-1)
  if (typeof left !== typeof right) {
    throw TYPE_MISMATCH
  }
  storeStack(-2, left === right)
  drop()
}

function dup() {
  assertStackHas(1)
  storeStack(0, readStack(-1))
}

function print() {
  assertStackHas(1)
  console.log(readStack(-1))
  drop()
}

function assert() {
  assertStackHas(2)
  const value = readStack(-2)
  const message = readStackString(-1)
  if (!value) {
    console.error("Assertion failed: ", message)
    throw ASSERT_FAILED
  }
  drop()
  drop()
}

function not() {
  assertStackHas(1)
  storeStack(-1, !readStack(-1))
}

function checkCondition() {
  assertStackHas(1)
  return !!STATE.values.pop()
}

function printStack() {
  if (STATE.values.length === 0) {
    return
  }
  console.log(STATE.values)
}

function to_char() {
  assertStackHas(1)
  const top = readStackString(-1)
  if (top.length !== 1) {
    throw TYPE_MISMATCH
  }
  storeStack(-1, top.charCodeAt(0))
}

function from_char() {
  assertStackHas(1)
  storeStack(-1, String.fromCharCode(readStackNumber(-1)))
}
