exception CorruptedFile of string

module CharSet = Set.Make (Char)

let () =
  let file_name = Sys.argv.(1) in
  let distinct_characters = int_of_string Sys.argv.(2) in
  let channel = open_in file_name in
  let set_of_list l =
    List.fold_left (fun s e -> CharSet.add e s) CharSet.empty l
  in
  let rec find_marker stream =
    let rec nchars s n =
      if n == 0 then []
      else
        let new_s = String.sub s 1 (String.length s - 1) in
        let chars = nchars new_s (n - 1) in
        s.[0] :: chars
    in
    let set = set_of_list (nchars stream distinct_characters) in
    if CharSet.cardinal set == distinct_characters then distinct_characters
    else
      let new_stream = String.sub stream 1 (String.length stream - 1) in
      find_marker new_stream + 1
  in
  let offset = find_marker (input_line channel) in
  Printf.printf "Part 1: the offset of the marker is %d\n" offset
