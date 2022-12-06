exception CorruptedFile of string

type assignment = { from_section : int; to_section : int }

let () =
  let file_name = Sys.argv.(1) in
  let channel = Scanf.Scanning.open_in file_name in
  let rec read_lines channel =
    try
      let f f1 t1 f2 t2 =
        let assignment1 = { from_section = f1; to_section = t1 } in
        let assignment2 = { from_section = f2; to_section = t2 } in
        (assignment1, assignment2)
      in
      let line = Scanf.bscanf channel "%i-%i,%i-%i\n" f in
      line :: read_lines channel
    with End_of_file -> []
  in
  let assignments = read_lines channel in
  let has_full_overlap a1 a2 =
    if
      (a1.from_section <= a2.from_section && a1.to_section >= a2.to_section)
      || (a2.from_section <= a1.from_section && a2.to_section >= a1.to_section)
    then true
    else false
  in
  let f overlap acc (a1, a2) = if overlap a1 a2 then acc + 1 else acc in
  let sum_overlaps = List.fold_left (f has_full_overlap) 0 assignments in
  Printf.printf "Part 1: There is %i overlapping assignments\n" sum_overlaps;
  let has_overlap a1 a2 =
    if a1.to_section < a2.from_section || a2.to_section < a1.from_section then
      false
    else true
  in
  let sum_overlaps = List.fold_left (f has_overlap) 0 assignments in
  Printf.printf "Part 2: There is %i overlapping assignments\n" sum_overlaps
