#let sign-digits(num, digits) = {
  if num == 0 {
    if digits == 1 {
      return $0$
    } else {
      return $0.#("0" * (digits - 1))$
    }
  }

  let sign = num.signum()
  num = calc.abs(num)

  let log = calc.floor(calc.log(num))
  let num = calc.round(num / calc.pow(10, log - (digits - 1)))
  if num >= calc.pow(10, digits) {
    num /= 10
    log += 1
  }

  let ret = if log + 1 >= digits {
    $num#("0" * (log - digits + 1))$
  } else if log >= 0 {
    $#(str(num).slice(0, log + 1)).#(str(num).slice(log + 1))$
  } else {
    $0.#("0" * (-log - 1))#num$
  }
  if sign == -1 {
    $-#ret$
  } else {
    ret
  }
}

#let rustd((secs, nanos)) = {
  secs + nanos * 1e-9
}