exception CorruptedFile of string
exception EmptyStack
exception EndShip

type crate = Crate of char | Empty
type move = { number : int; src : int; dst : int }

class stack =
  object (self)
    val mutable inner : crate list = []

    method insert c =
      match c with
      | Crate c -> inner <- inner @ [ Crate c ]
      | Empty -> inner <- inner

    method put c =
      match c with
      | Crate c -> inner <- Crate c :: inner
      | Empty -> inner <- inner

    method putn cs =
      match cs with
      | [] -> ()
      | c :: r -> (
          self#putn r;
          match c with Crate _ -> self#put c | Empty -> ())

    method pop =
      match inner with
      | c :: r ->
          inner <- r;
          c
      | [] -> raise EmptyStack

    method popn n =
      match n with
      | 0 -> raise (Invalid_argument "Need to 'popn' at least one crate")
      | 1 -> [ self#pop ]
      | _ ->
          if n < 0 then
            raise (Invalid_argument "Cannot 'popn' a negative number of crates")
          else
            let top = self#pop in
            let bottom = self#popn (n - 1) in
            top :: bottom

    method top =
      match inner with
      | crate :: _ ->
          let c = match crate with Crate c -> Char.escaped c | Empty -> "*" in
          c
      | [] -> "_"
  end

class ship =
  object (self)
    val mutable stacks : stack list = []
    method length = List.length stacks

    method insert idx crate =
      let insert_stack i stack =
        if i == idx then stack#insert crate;
        stack
      in
      if self#length <= idx then stacks <- List.append stacks [ new stack ];
      stacks <- List.mapi insert_stack stacks

    method put idx crate =
      let put_stack i stack =
        if i == idx then stack#put crate;
        stack
      in
      stacks <- List.mapi put_stack stacks

    method putn idx crates =
      let putn_stack i stack =
        if i == idx then stack#putn crates;
        stack
      in
      stacks <- List.mapi putn_stack stacks

    method pop idx =
      let stack = List.nth stacks idx in
      stack#pop

    method popn idx n =
      let stack = List.nth stacks idx in
      stack#popn n

    method move m =
      if m.number > 0 then (
        let crate = self#pop (m.src - 1) in
        self#put (m.dst - 1) crate;
        let move = { number = m.number - 1; src = m.src; dst = m.dst } in
        self#move move)

    method moven m =
      let crates = self#popn (m.src - 1) m.number in
      self#putn (m.dst - 1) crates

    method tops =
      let topi tops stack = tops ^ stack#top in
      List.fold_left topi "" stacks
  end

let () =
  let file_name = Sys.argv.(1) in
  let is_crate_mover_9001 = bool_of_string Sys.argv.(2) in
  let channel = open_in file_name in
  let ship = new ship in
  let rec read_ship channel stack_idx =
    let crate_builder c1 c2 c3 =
      match (c1, c2, c3) with
      | '[', c, ']' -> Crate c
      | ' ', ' ', ' ' -> Empty
      | ' ', '1', ' ' -> raise EndShip
      | _ -> raise (CorruptedFile file_name)
    in
    let c1 = input_char channel in
    let c2 = input_char channel in
    let c3 = input_char channel in
    let crate = crate_builder c1 c2 c3 in
    ship#insert stack_idx crate;
    let c4 = input_char channel in
    match c4 with
    | ' ' -> read_ship channel (stack_idx + 1)
    | '\n' -> read_ship channel 0
    | _ -> raise (CorruptedFile file_name)
  in
  try read_ship channel 0
  with EndShip -> (
    let _ = input_line channel in
    let _ = input_line channel in
    try
      let rec read_moves channel =
        let line = input_line channel in
        let to_move number src dst = { number; src; dst } in
        let m = Scanf.sscanf line "move %d from %d to %d" to_move in
        if is_crate_mover_9001 then ship#moven m else ship#move m;
        read_moves channel
      in
      read_moves channel
    with End_of_file ->
      Printf.printf "Part 1: the top crates are %s\n" ship#tops)
