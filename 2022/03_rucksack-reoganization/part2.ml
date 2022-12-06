exception CorruptedFile of string

let () =
  let file_name = Sys.argv.(1) in
  let channel = open_in file_name in
  let rec read_lines channel =
    try
      let first = input_line channel in
      let second = input_line channel in
      let third = input_line channel in
      (first, second, third) :: read_lines channel
    with End_of_file -> []
  in
  let rucksacks = read_lines channel in
  let find (first, second, third) =
    let find_c c s1 s2 =
      let i1 = String.index_opt s1 c in
      let i2 = String.index_opt s2 c in
      if Option.is_some i1 && Option.is_some i2 then Some c else None
    in
    let f acc c = if Option.is_some acc then acc else find_c c second third in
    String.fold_left f None first
  in
  let badges_opt = List.map find rucksacks in
  let unwrap c_opt =
    match c_opt with Some c -> c | None -> raise (CorruptedFile file_name)
  in
  let badges = List.map unwrap badges_opt in
  let score_badge c =
    if Char.equal c (Char.lowercase_ascii c) then
      Char.code c - Char.code 'a' + 1
    else Char.code c - Char.code 'A' + 27
  in
  let scores = List.map score_badge badges in
  let sum = List.fold_left ( + ) 0 scores in
  Printf.printf "Part 2: the total score is %d\n" sum
