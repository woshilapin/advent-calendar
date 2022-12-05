exception CorruptedFile

let () =
  let file_name = Sys.argv.(1) in
  let channel = open_in file_name in
  let rec read_lines channel =
    try
      let line = input_line channel in
      let split = String.split_on_char ' ' line in
      let first, second =
        match split with
        | [ first; second ] -> (first, second)
        | _ -> raise CorruptedFile
      in
      (first, second) :: read_lines channel
    with End_of_file -> []
  in
  let matches = read_lines channel in
  let part1 acc m =
    let shape_score =
      match m with
      | _, "X" -> 1
      | _, "Y" -> 2
      | _, "Z" -> 3
      | _ -> raise CorruptedFile
    in
    let game_score =
      match m with
      | "A", "X" | "B", "Y" | "C", "Z" -> 3
      | "A", "Z" | "B", "X" | "C", "Y" -> 0
      | "A", "Y" | "B", "Z" | "C", "X" -> 6
      | _ -> raise CorruptedFile
    in
    acc + shape_score + game_score
  in
  let result = List.fold_left part1 0 matches in
  print_string "Part 1: following the strategy guide, you scored ";
  print_int result;
  print_endline "";
  let part2 acc m =
    let shape_score =
      match m with
      | "A", "X" | "C", "Y" | "B", "Z" -> 3
      | "B", "X" | "A", "Y" | "C", "Z" -> 1
      | "C", "X" | "B", "Y" | "A", "Z" -> 2
      | _ -> raise CorruptedFile
    in
    let game_score =
      match m with
      | _, "X" -> 0
      | _, "Y" -> 3
      | _, "Z" -> 6
      | _ -> raise CorruptedFile
    in
    acc + shape_score + game_score
  in
  let result = List.fold_left part2 0 matches in
  print_string "Part 2: following the strategy guide, you scored ";
  print_int result;
  print_endline ""
