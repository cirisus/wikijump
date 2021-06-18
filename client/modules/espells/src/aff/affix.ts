import iterate from "iterare"
import type { Aff } from "."
import { re } from "../util"

abstract class Affix {
  declare flag: string
  declare crossproduct: boolean
  declare strip: string
  declare add: string
  declare condition: string
  declare flags: Set<string>

  declare conditionRegex: RegExp
  declare lookupRegex: RegExp
  declare replaceRegex: RegExp

  constructor(
    flag: string,
    crossproduct: string,
    strip: string,
    add: string,
    condition: string,
    aff: Aff
  ) {
    let flags: string
    ;[add, flags] = add.split("/")

    if (aff.IGNORE) {
      for (const ch of aff.IGNORE) {
        add = add.replaceAll(ch, "")
      }
    }

    this.flag = flag
    this.crossproduct = crossproduct === "Y"
    this.strip = strip === "0" ? "" : strip
    this.add = add === "0" ? "" : add
    this.condition = condition
    this.flags = flags ? aff.parseFlags(flags) : new Set()
  }
}

export class Prefix extends Affix {
  constructor(
    flag: string,
    crossproduct: string,
    strip: string,
    add: string,
    condition: string,
    aff: Aff
  ) {
    super(flag, crossproduct, strip, add, condition, aff)

    let parts = iterate(condition.matchAll(/(\[.+\]|[^\[])/g))
      .map(part => part.slice(1))
      .flatten()
      .toArray()

    if (parts.length && this.strip) parts = parts.slice(this.strip.length)

    let cond = ""

    if (parts.length && !(parts.length === 1 && parts[0] === ".")) {
      cond = `(?=${parts.join("")})`.replaceAll("-", "\\-")
    }

    this.conditionRegex = re`/^${this.condition.replaceAll("-", "\\-")}/`
    this.lookupRegex = re`/^${this.add}${cond}/`
    this.replaceRegex = re`/^${this.add}/`
  }
}

export class Suffix extends Affix {
  constructor(
    flag: string,
    crossproduct: string,
    strip: string,
    add: string,
    condition: string,
    aff: Aff
  ) {
    super(flag, crossproduct, strip, add, condition, aff)

    let parts = iterate(condition.matchAll(/(\[.+\]|[^\[])/g))
      .map(part => part.slice(1))
      .flatten()
      .toArray()

    let cond = ""

    if (parts.length && !(parts.length === 1 && parts[0] === ".")) {
      if (this.strip) parts = parts.slice(0, -this.strip.length)
      cond = `(${parts.join("")})`.replaceAll("-", "\\-")
    }

    this.conditionRegex = re`/${this.condition.replaceAll("-", "\\-")}$/`
    this.lookupRegex = re`/${cond}${this.add}$/`
    this.replaceRegex = re`/${this.add}$/`
  }
}
