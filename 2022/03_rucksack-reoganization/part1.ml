exception CorruptedFile of string

let () =
  let file_name = Sys.argv.(1) in
  let channel = open_in file_name in
  let rec read_lines channel =
    try
      let line = input_line channel in
      line :: read_lines channel
    with End_of_file -> []
  in
  let rucksacks = read_lines channel in
  let split line =
    let l = String.length line in
    let m = l / 2 in
    let first = String.sub line 0 m in
    let second = String.sub line m m in
    (first, second)
  in
  let rucksacks = List.map split rucksacks in
  let score (first, second) =
    let f acc c =
      if Option.is_some acc then acc
      else
        let index = String.index_opt second c in
        if Option.is_some index then Some c else None
    in
    let c_opt = String.fold_left f Option.None first in
    let c =
      match c_opt with Some c -> c | None -> raise (CorruptedFile file_name)
    in
    let value =
      if Char.equal c (Char.lowercase_ascii c) then
        Char.code c - Char.code 'a' + 1
      else Char.code c - Char.code 'A' + 27
    in
    value
  in
  let scores = List.map score rucksacks in
  let sum = List.fold_left ( + ) 0 scores in
  Printf.printf "Part 1: the total score is %d\n" sum
