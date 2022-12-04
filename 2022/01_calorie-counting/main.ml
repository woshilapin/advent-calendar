let file = "calories.txt"

exception MissingElves of int

let () =
  let channel = open_in file in
  print_endline "";
  let rec read_lines () =
    try
      let line = input_line channel in
      line :: read_lines ()
    with End_of_file -> []
  in
  let to_int line = int_of_string_opt line in
  let calories_str = read_lines () in
  let calories = List.map to_int calories_str in
  let rec group_calories calories =
    match calories with
    | Some c1 :: Some c2 :: r -> group_calories (Some (c1 + c2) :: r)
    | Some c :: None :: r -> Some c :: None :: group_calories r
    | None :: r -> group_calories r
    | Some _ :: [] | [] -> calories
  in
  let grouped_calories = group_calories calories in
  let rec unwrap l =
    match l with
    | [] -> []
    | Some c :: r -> c :: unwrap r
    | None :: r -> unwrap r
  in
  let unwrapped = unwrap grouped_calories in
  let sorted = List.fast_sort (fun a b -> b - a) unwrapped in
  let first, second, third =
    match sorted with
    | first :: second :: third :: _ -> (first, second, third)
    | _ -> raise (MissingElves (List.length sorted))
  in
  print_string "Part 1: The Elf carrying the most calories has ";
  print_int first;
  print_endline " calories";
  print_string
    "Part 2: The 3 Elves with the most calories are respectively carrying ";
  print_int first;
  print_string ", ";
  print_int second;
  print_string " and ";
  print_int third;
  print_string " calories";
  print_string " for a total of ";
  print_int (first + second + third);
  print_endline " calories"
