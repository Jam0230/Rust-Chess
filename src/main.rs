use std::fs;
use std::cmp;
use std::io;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PieceType{
	Pawn,
	Rook,
	Knight,
	Bishop,
	Queen,
	King,
	None
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PieceColour{
	White,
	Black,
	None
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Piece{
	piece_type: PieceType,
	piece_colour: PieceColour
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MoveFlag{ // flags for special moves
	EnPassant,
	Castling,
	
	Promotion, // general promotion flag
	// promotion flags added later when type of promotion is chosen 
	RookPromo,
	KnightPromo,
	BishopPromo,
	QueenPromo,
	None
}

#[derive(Debug, Copy, Clone)]
struct Move{
	start: i32,
	end: i32,
	flag: MoveFlag
}

// ------- 	FEN STUFF -------

fn decode_fen(fen_string: &str) -> (Vec<Piece>, PieceColour, (bool,bool,bool,bool), i32){ // returns board state given by a fen string
	let fen_parts: Vec<&str> = fen_string.split(" ").collect();
	let mut return_tuple: (Vec<Piece>, PieceColour, (bool,bool,bool,bool), i32) = (Vec::new(), PieceColour::None, (false, false, false, false), -1); // board, colours turn, en passant move, castling rights

	// -- PIECE PLACEMENT -- 

	let ranks = fen_parts[0].split("/"); // first part of the fen string (piece placement)
	let mut board: Vec<Piece> = Vec::new();

	for rank in ranks{
		for file in rank.chars(){
			if file.is_numeric(){ // empty space
				for _ in 0..file.to_digit(10).unwrap(){ // add multiple empty spaces to board
					board.push(Piece{ piece_type: PieceType::None, piece_colour: PieceColour::None});
				}
			} else{ // piece
				let mut piece_type = PieceType::None;
				let piece_colour;

				match file.to_lowercase().to_string().as_str(){
					"p" => piece_type = PieceType::Pawn, // pawn (p)
					"r" => piece_type = PieceType::Rook, // rook (r)
					"n" => piece_type = PieceType::Knight, // knight (n)
					"b" => piece_type = PieceType::Bishop, // bishop (b)
					"q" => piece_type = PieceType::Queen, // queen (q)
					"k" => piece_type = PieceType::King, // king (k)
					_ => {println!("\x1b[41m-UNEXPECTED VALUE IN FEN STRING--\x1b[0m"); return_tuple.0 = Vec::new(); } // not piece character in fen string (e.g: d)
				}

				match file.is_uppercase(){
					true => piece_colour = PieceColour::White, // white (uppercase)
					false => piece_colour = PieceColour::Black, // black (lowercase)
				}

				board.push(Piece { piece_type: piece_type, piece_colour: piece_colour}); // add piece to board
			}
		}
	}
	if board.len() == 64{
		return_tuple.0 = board;
	}
	else{
		println!("\x1b[41m--UNEXPECTED LENGTH FEN STRING--\x1b[0m");
		return_tuple.0 = Vec::new();
	}


	// -- COLOURS TURN --

	match fen_parts[1]{
		"w" => return_tuple.1 = PieceColour::White,
		"b" => return_tuple.1 = PieceColour::Black,
		_ => { println!("\x1b[41m--UNEXPECTED VALUE IN FEN STRING--\x1b[0m"); return_tuple.0 = Vec::new(); }
	}

	// -- CASTLING RIGHTS --

	let castling_rights_string = fen_parts[2];

	for castling_char in castling_rights_string.chars(){
		match castling_char{
			'K' => return_tuple.2.0 = true, // white king side
			'Q' => return_tuple.2.1 = true, // white queen side
			'k' => return_tuple.2.2 = true, // black king side
			'q' => return_tuple.2.3 = true, // black queen side
			'-' => return_tuple.2 = (false,false,false,false),
			_ => { println!("\x1b[41m--UNEXPECTED VALUE IN FEN STRING--\x1b[0m"); return_tuple.0 = Vec::new(); }
		}
	}

	// -- EN PASSANT TARGET -- 

	let en_passant_target_algebraic = fen_parts[3];
	let mut en_passant_target = -1;

	if en_passant_target_algebraic != "-"{
		let number_part = en_passant_target_algebraic.chars().nth(1).expect("\x1b[41m--UNEXPECTED VALUE IN FEN STRING--\x1b[0m").to_digit(10).unwrap();
		let letter_part = en_passant_target_algebraic.chars().nth(0).expect("\x1b[41m--UNEXPECTED VALUE IN FEN STRING--\x1b[0m");

		match letter_part.to_lowercase().to_string().as_str(){
			"a" => en_passant_target += 0,
			"b" => en_passant_target += 1,
			"c" => en_passant_target += 2,
			"d" => en_passant_target += 3,
			"e" => en_passant_target += 4,
			"f" => en_passant_target += 5,
			"g" => en_passant_target += 6,
			"h" => en_passant_target += 7,
			_ => { println!("\x1b[41m--UNEXPECTED VALUE IN FEN STRING--\x1b[0m"); return_tuple.0 = Vec::new(); }
		}
		en_passant_target += (8-number_part as i32)*8;
	}

	return_tuple.3 = en_passant_target;

	return return_tuple
}

fn load_board_art(file_path: &str) -> Vec<[String;9]>{ // loads art used when printing board from file
	let file_contents = fs::read_to_string(file_path).expect("\x1b[41m--COULD NOT READ ART FILE--\x1b[0m");
	let split_contents = file_contents.split("\n"); // Read then split art file contents
	let mut piece_arts: Vec<[String;9]> = Vec::new();

	let mut n = 0;
	let mut piece_art: [String;9] = Default::default(); // initialises the piece art array (9 * "".to_string())
	for line in split_contents{
		if n == 8{
			piece_art[n] = line.to_string(); // add last line to piece art
			piece_arts.push(piece_art.clone()); // add piece art to list of arts
			n=0;
		}
		else{
			piece_art[n] = line.to_string(); // add line to piece art
			n += 1;
		}
	}
	piece_arts
}

fn encode_into_fen(board: &Vec<Piece>, colours_turn: PieceColour, castling_rights: (bool,bool,bool,bool), en_passant_move: i32) -> String{ // returns fen string of current boardkkkkll
	let mut fen_string = String::new();

	// -- BOARD LAYOUT --

	let mut board_layout_str = String::new();

	let mut empty_flag = false;
	let mut num_empty = 0;

	for rank in 0..8{
		for file in 0..8{
			let piece = board[file+rank*8];

			if empty_flag == true && piece.piece_type != PieceType::None{
				board_layout_str.push_str(&num_empty.to_string());
				empty_flag = false;
				num_empty = 0;
			}

			match piece.piece_type{
				PieceType::Pawn => { // pawn
					match piece.piece_colour{
						PieceColour::White => board_layout_str.push_str("P"), // white
						PieceColour::Black => board_layout_str.push_str("p"), // black
						_ => ()
					}
				},
				PieceType::Rook => { // rook
					match piece.piece_colour{
						PieceColour::White => board_layout_str.push_str("R"), // white
						PieceColour::Black => board_layout_str.push_str("r"), // black
						_ => ()						
					}
				},
				PieceType::Knight => { // knight
					match piece.piece_colour{
						PieceColour::White => board_layout_str.push_str("N"), // white
						PieceColour::Black => board_layout_str.push_str("n"), // black
						_ => ()						
					}
				},
				PieceType::Bishop => { // bishop
					match piece.piece_colour{
						PieceColour::White => board_layout_str.push_str("B"), // white
						PieceColour::Black => board_layout_str.push_str("b"), // black
						_ => ()						
					}
				},
				PieceType::Queen => { // queen
					match piece.piece_colour{
						PieceColour::White => board_layout_str.push_str("Q"), // white
						PieceColour::Black => board_layout_str.push_str("q"), // black
						_ => ()						
					}
				},
				PieceType::King => { // king
					match piece.piece_colour{
						PieceColour::White => board_layout_str.push_str("K"), // white
						PieceColour::Black => board_layout_str.push_str("k"), // black
						_ => ()						
					}
				},
				PieceType::None => { // empty square
					empty_flag = true;
					num_empty += 1;
				}
			}
		}

		if num_empty != 0{ // adding empty count if whole rank is empty
			board_layout_str.push_str(&num_empty.to_string());
			num_empty = 0;
			empty_flag = false;
		}

		if rank != 7 { // adding rank seperators
			board_layout_str.push_str("/")
		}
	}

	fen_string.push_str(&(board_layout_str+" "));


	// -- COLOURS TURN -- 

	match colours_turn {
		PieceColour::White => fen_string.push_str("w "),
		PieceColour::Black => fen_string.push_str("b "),
		_ => (),
	}

	// -- CASTLING RIGHTS -- 

	let mut castling_rights_string = String::new();

	if castling_rights.0 == true{
		castling_rights_string.push_str("K"); // white king side
	}
	if castling_rights.1 == true{
		castling_rights_string.push_str("Q"); // white queen side
	}
	if castling_rights.2 == true{
		castling_rights_string.push_str("k"); // black king side
	}
	if castling_rights.3 == true{
		castling_rights_string.push_str("q"); // black queen side
	}

	if castling_rights_string.len() == 0{ // if no castling is available
		castling_rights_string.push_str("- ");
	}
	else{
		castling_rights_string.push_str(" ");
	}

	fen_string.push_str(&castling_rights_string);

	// -- EN PASSANT MOVE --

	if en_passant_move == -1{
		fen_string.push_str("- ");
	}
	else{

		let letter_part = match en_passant_move%8{
			0 => "a",
			1 => "b",
			2 => "c",
			3 => "d",
			4 => "e",
			5 => "f",
			6 => "g",
			7 => "h",
			_ => "-",
		};

		let number_part = (8-(en_passant_move/8)).to_string();

		let algebraic_notation_input = format!("{}{} ", letter_part, number_part);

		fen_string.push_str(&algebraic_notation_input);

	}

	println!("{}", fen_string);

	fen_string
}

// ------- BOARD PRINTING -------

fn print_board(board: &Vec<Piece>, piece_moves: &Vec<Move>, piece_art: &Vec<[String;9]>){ // prints the board fancily
	let mut lines: Vec<String> = vec![String::new(); 9];

	for rank in 0..=8{
		let mut art_index: i32;

		match rank{ // numbers on the left
			0 => art_index = 22, // 8
			1 => art_index = 21, // 7
			2 => art_index = 20, // 6
			3 => art_index = 19, // 5
			4 => art_index = 18, // 4
			5 => art_index = 17, // 3
			6 => art_index = 16, // 2
			7 => art_index = 15, // 1
			_ => art_index = 0 // Blank
		}

		for i in 0..=8{ // placing numbers
			let line = piece_art[art_index as usize][i].clone().replace("#", " ");

			lines[i] = format!("{}{}", lines[i], line);
		}

		for file in 0..=7{

			if rank == 8{ // letters at bottom
				match file{
					0 => art_index = 7, // A
					1 => art_index = 8, // B
					2 => art_index = 9, // C
					3 => art_index = 10, // D
					4 => art_index = 11, // E
					5 => art_index = 12, // F
					6 => art_index = 13, // G
					7 => art_index = 14, // H
					_ => art_index = 0, // Blank (Should never happen but rust wants to make me do it)
				}

				for i in 0..=8{ 
					let line = piece_art[art_index as usize][i].clone().replace("#", " "); //

					lines[i] = format!("{}{}", lines[i], line);
				}

			}else{ // actual pieces
				let index = file+rank*8;

				match board[index].piece_type{
					PieceType::Pawn => art_index = 1, // Pawn
					PieceType::Rook => art_index = 2, // Rook
					PieceType::Knight => art_index = 3, // Knight
					PieceType::Bishop => art_index = 4, // Bishop
					PieceType::Queen => art_index = 5, // Queen
					PieceType::King => art_index = 6, // King
					PieceType::None => art_index = 0 // Empty
				}

				for i in 0..=8{
					let mut line = piece_art[art_index as usize][i].clone();

					match (rank+1)%2 != (file+1)%2{
						true => line = line.replace("#", " "), // Replace '#' with ' ' if Black Square
						false => () // Dont change if White Square
					}

					match board[index].piece_colour{
						PieceColour::Black => line = line.replace("■", "\x1b[30m■\x1b[0m"), // Replace white square with black square if black piece
						_ => () // Dont change if White Piece/Empty space
					}

					for piece_move in piece_moves.into_iter(){
						if index as i32 == piece_move.end{
							if board[index].piece_type != PieceType::None{
								line = line.replace("#", "\x1b[32m@\x1b[0m"); // Highlight green if piece
								line = line.replace(" ", "\x1b[32m*\x1b[0m");
							}
							else{
								line = line.replace("#", "\x1b[33m@\x1b[0m"); // Hightlight yellow if not
								line = line.replace(" ", "\x1b[33m*\x1b[0m");
							}
						}
					}

					lines[i] = format!("{}{}", lines[i], line); // Add line to output
				}
			}
		}

		for line in lines{
			println!("{}", line); // Output lines
		}
		lines = vec![String::new(); 9];
	}
}

// ------- PIECE MOVE GENERATION -------

fn dist_to_edge(index: i32) -> [i32; 8] { // finds distance to edge in each direction
	let file = index % 8;
	let rank = index / 8;

	let south = 7-rank; // down
	let north = rank; // up
	let east = 7-file; // right
	let west = file; // left

	let south_east = cmp::min(south, east); // down right
	let north_west = cmp::min(north, west); // up left
	let south_west = cmp::min(south, west); // down left
	let north_east = cmp::min(north, east); // up right

	[south, north, east, west, south_east, north_west, south_west, north_east]
}

fn legal_move_gen(board: &mut Vec<Piece>, index: i32, en_passant_move: i32, king_indexs: (i32, i32), castling_rights: (bool,bool,bool,bool)) -> Vec<Move>{ // removes any piece moves that result in check
	let sudo_legal_moves = sudo_legal_move_gen(board, index, en_passant_move, castling_rights, true); // all possible moves that can be made by the piece
	let mut legal_moves: Vec<Move> = Vec::new();

	for sudo_move in sudo_legal_moves{

		let (board_after_move, new_en_passant_move, _, _) = make_move(&mut board.clone() /*need the .clone() there to stop it from editing the actual board*/, sudo_move, en_passant_move, king_indexs, castling_rights); 
			// sudo make move on board
		let opponent_colour: PieceColour;  // colour off opponents piece
		let mut king_index: i32; // index of current players king

		match board[index as usize].piece_colour{ // set the opponent colour and king index
			PieceColour::White => { opponent_colour = PieceColour::Black; king_index = king_indexs.0},
			PieceColour::Black => { opponent_colour = PieceColour::White; king_index = king_indexs.1},
			_ => { println!("\x1b[41m--UNEXPECTED PIECE COLOUR WHEN GENERATING MOVES--\x1b[0m"); return Vec::new(); } // return error if currently checked piece is blank space
		}

		if board[index as usize].piece_type == PieceType::King{ // if the piece moving is the king move the king index with the move being checked  (stops king blocking check on itself)
			king_index = sudo_move.end;
		}

		let opponent_responses = side_move_gen(&board_after_move, new_en_passant_move, opponent_colour, castling_rights); // possible moves the opponent can make (check not respected)

		// if any of the opponent responses results in king being taken then the move is not legal
		let mut illigal_flag = false; 
		for opponent_move in opponent_responses{
			if opponent_move.end == king_index{ 
				illigal_flag = true;
			}
		}

		if illigal_flag == false{ // add the move to the legal moves if no responces result in the king being taken
			legal_moves.push(sudo_move);
		}

	}

	legal_moves
}

fn sudo_legal_move_gen(board: &Vec<Piece>, index: i32, en_passant_move: i32, castling_rights: (bool,bool,bool,bool), check_castling: bool) -> Vec<Move>{ // generates moves without respect to check
	let mut piece_moves: Vec<Move> = Vec::new();
	let piece = board[index as usize];

	match piece.piece_type{
		PieceType::Rook | PieceType::Bishop | PieceType::Queen => { // Rook Bishop Queen
			let directions = [8, -8, 1, -1, 9, -9, 7, -7]; // south north east west south-east north-west south-west north-east
			let edge_dist = dist_to_edge(index); // Dist to edges in each direction

			//Cutting off directions piece cant move in
			let start_dir = if piece.piece_type == PieceType::Bishop {4} else {0};
			let end_dir = if piece.piece_type == PieceType::Rook {4} else {8};

			for dir_index in start_dir..end_dir{
				for dir_offset in 1..edge_dist[dir_index]+1{
					let dir = directions[dir_index];

					let new_index = index + (dir * dir_offset);

					if board[new_index as usize].piece_colour == piece.piece_colour{ break; } // target piece same colour

					piece_moves.push(Move{ start: index, end: new_index , flag: MoveFlag::None});

					if board[new_index as usize].piece_colour != PieceColour::None{ break; } // target piece enemy colour
				}
			}
		},
		PieceType::Knight => {
			let moves = [(-2,-1), (-1,-2), (1,-2), (2,-1), (2,1), (1,2), (-1,2), (-2,1)]; // Moves possible by knight

			for knight_move in moves{
				let new_file = (index%8) + knight_move.0;
				let new_rank = (index/8) + knight_move.1;

				if 0>new_file || 7<new_file || 0>new_rank || 7<new_rank { continue; } // Removing moves that go off the board

				if board[(new_file+new_rank*8) as usize].piece_colour == piece.piece_colour { continue; } // Removing moves that are targeting pieces of the same colour

				piece_moves.push(Move{ start: index, end: new_file+new_rank*8, flag: MoveFlag::None})
			}
		},
		PieceType::Pawn => {
			let mut dir = -1; // direction of travel
			let mut start_rank = 6; // rank on which the pawn starts
			let mut promotion_rank = 0;
			let mut en_passant_rank = 3; // rank on which the pawn can perform en passant
			if piece.piece_colour == PieceColour::Black { dir=1; start_rank=1; promotion_rank=7; en_passant_rank=4; } // black pawn values

			if board[(index+(8*dir)) as usize].piece_colour == PieceColour::None{ // single move forward
				if (index+(8*dir))/8 != promotion_rank{
					piece_moves.push(Move{ start: index, end: index+(8*dir), flag: MoveFlag::None}); // not last rank 
				} else {
					piece_moves.push(Move{ start: index, end: index+(8*dir), flag: MoveFlag::Promotion}); // last rank so promotion 
				}

				if index+(16*dir) >= 0 && index+(16*dir) <= 63{
					if board[(index+(16*dir)) as usize].piece_colour == PieceColour::None && index/8 == start_rank{ // double move forward when on starting rank
						piece_moves.push(Move{ start: index, end: index+(16*dir), flag: MoveFlag::None});
					}
				}
			}

			for side in [-1i32, 1]{
				if 0>(index%8)+side || 7<(index%8)+side{ continue; } // removing moves that go off the board

				if board[(index+side+(8*dir)) as usize].piece_colour != piece.piece_colour && board[(index+side+(8*dir)) as usize].piece_colour != PieceColour::None{ // taking diagonally on each side
					if (index+side+(8*dir))/8 != promotion_rank{
						piece_moves.push(Move{ start: index, end: index+side+(8*dir), flag: MoveFlag::None});// not last rank 
					} else {
						piece_moves.push(Move{ start: index, end: index+side+(8*dir), flag: MoveFlag::Promotion}); // last rank so promotion 
					}
				}

				if en_passant_move == index+side+(8*dir) && board[(index+side) as usize].piece_colour != piece.piece_colour && index/8 == en_passant_rank{ // en passant
					piece_moves.push(Move{ start: index, end: index+side+(8*dir), flag: MoveFlag::EnPassant})
				}
			}

		},
		PieceType::King => {
			for x_change in -1..=1{
				for y_change in -1..=1{
					if x_change != 0 || y_change != 0{
						let new_file = (index%8)+x_change;
						let new_rank = (index/8)+y_change;

						if 0>new_file || 7<new_file || 0>new_rank || 7<new_rank { continue; } // removing moves that go off the board

						if board[(new_file+new_rank*8) as usize].piece_colour != piece.piece_colour{ // removing moves that are targeting a piece of the same colour
							piece_moves.push(Move{ start: index, end: new_file+new_rank*8, flag: MoveFlag::None});
						}
					}
				}
			}

			// castling stuff
			if check_castling{

				let opponent_moves = match piece.piece_colour{
					PieceColour::White => side_move_gen(board, en_passant_move, PieceColour::Black, castling_rights),
					PieceColour::Black => side_move_gen(board, en_passant_move, PieceColour::White, castling_rights),
					_ => Vec::new()
				};

				'stuart :for dir in [-1i32, 1i32]{
					let mut blocking_indexs: Vec<i32> = Vec::new();
					match dir {
						-1 => { // queen side
							let can_castle = match piece.piece_colour{
								PieceColour::White => castling_rights.1,
								PieceColour::Black => castling_rights.3,
								_ => false
							};

							blocking_indexs = vec![index-1, index-2, index-3];

							if !can_castle { continue; }
						},
						1 => { // king side
							let can_castle = match piece.piece_colour{
								PieceColour::White => castling_rights.0,
								PieceColour::Black => castling_rights.2,
								_ => false
							};

							blocking_indexs = vec![index+1, index+2];

							if !can_castle { continue; }
						},
						_ => ()
					}

					for index_to_check in blocking_indexs{
						if board[index_to_check as usize].piece_type != PieceType::None{
							continue 'stuart;
						}

						for enemy_move in &opponent_moves{
							if enemy_move.end == index_to_check{
								continue 'stuart;
							}
						}
					}

					piece_moves.push(Move{ start: index, end: index+(2*dir), flag: MoveFlag::Castling});
				}
			}
		},
		_ => return Vec::new()
	}
	piece_moves
}

fn side_move_gen(board: &Vec<Piece>, en_passant_move: i32, side_to_check: PieceColour, castling_rights: (bool,bool,bool,bool)) -> Vec<Move>{ // generates all moves for a side 
	let mut moves: Vec<Move> = Vec::new();

	for piece_index in 0..64{ // loops through all pieces on the board

		if board[piece_index].piece_colour == side_to_check{ // if piece is colour you want to check add piece moves to list
			moves.append(&mut sudo_legal_move_gen(board, piece_index as i32, en_passant_move, castling_rights, false));
		}
	}

	moves
}

// ------- PIECE MOVEMENT -------

fn selection_iteration(mut board: Vec<Piece>, mut colours_turn: PieceColour, en_passant_move: i32, piece_arts: &Vec<[String;9]>, king_indexs: (i32, i32), castling_rights: (bool,bool,bool,bool)) 
	-> (Vec<Piece>, i32, PieceColour, (i32, i32), (bool,bool,bool,bool)){ // the main input loop of the game
	let mut piece_moves: Vec<Move>;
	let mut selected_move: Move;

	loop{
		print_board(&board, &Vec::new(), piece_arts); // print current positions
		piece_moves = select_piece(&mut board, colours_turn, en_passant_move, king_indexs, castling_rights); // select piece

		print_board(&board, &piece_moves, piece_arts); // print piece moves
		selected_move = select_move(&piece_moves); // select move

		if selected_move.start != selected_move.end{
			break;
		}	
	}

	match colours_turn{ // swap whos turn it is
		PieceColour::White => colours_turn = PieceColour::Black,
		PieceColour::Black => colours_turn = PieceColour::White,
		_ => (),
	}

	let (new_board, en_passant_move, king_indexs, castling_rights) = make_move(&mut board, selected_move, en_passant_move, king_indexs, castling_rights);
	(new_board, en_passant_move, colours_turn, king_indexs, castling_rights) // make move and return new board
}

fn select_piece(board: &mut Vec<Piece>, colours_turn: PieceColour, en_passant_move: i32, king_indexs: (i32, i32), castling_rights: (bool,bool,bool,bool)) -> Vec<Move> { // returns moves of selected piece
	let mut index: i32;
	let mut piece_moves: Vec<Move>;

	loop{
		println!("{:?}'s turn!", colours_turn);
		index = algebraic_notation_input("Enter the piece you would like to select", false);

		if board[index as usize].piece_colour == colours_turn{ // check if piece selected is current person piece
			piece_moves = legal_move_gen(board, index, en_passant_move, king_indexs, castling_rights); 

			if piece_moves.len() > 0 { // make sure the piece has atleast one move
				break;
			}
			else{
				println!("-- Piece has no moves to make! --");
			}
		}
		else{
			println!("-- Not Your Piece! --");
		}
	}
	piece_moves
}

fn select_move(piece_moves: &Vec<Move>) -> Move{ // returns selected move
	loop{
		let index = algebraic_notation_input("Enter the move you would like to make (enter 'quit' to return to piece selection)", true);

		if index == -1{
			return Move{ start: -1, end: -1, flag: MoveFlag::None}; // go back to piece input
		}

		for piece_move in piece_moves.into_iter(){
			if piece_move.end == index{
				if piece_move.flag == MoveFlag::Promotion{
					let promo_flag = promotion_type_input("Enter the type of piece this pawn should promote to (enter 'quit' to return to piece selection)", true);

					if promo_flag == MoveFlag::None{ // go back to piece input
						return Move{ start: -1, end: -1, flag: MoveFlag::None};
					}

					return Move{ start: piece_move.start, end: piece_move.end, flag: promo_flag}; // give correct flag to move 
				}

				return *piece_move
			}
		}

		println!("-- Not a move this piece can make! --")
	}
}

fn make_move(board: &mut Vec<Piece>, piece_move: Move, en_passant_move: i32, mut king_indexs: (i32, i32), mut castling_rights: (bool,bool,bool,bool)) -> (Vec<Piece>, i32, (i32, i32), (bool,bool,bool,bool)){ // returns new board state for move made
	let start_piece = board[piece_move.start as usize];
	let capture_piece = board[piece_move.end as usize];
	let mut new_en_passant = -1;

	board[piece_move.end as usize] = start_piece; // move piece to new square
	board[piece_move.start as usize] = Piece{ piece_type: PieceType::None, piece_colour: PieceColour::None}; // remove piece at old position

	let mut pawn_dir = -1; // direction of pawn travel
	let mut pawn_start = 6; // start rank of pawn
	if start_piece.piece_colour == PieceColour::Black { pawn_dir = 1; pawn_start = 1;}; // black pawn values


	// -- en passant stuff -- 

	if piece_move.flag == MoveFlag::EnPassant{
		board[(en_passant_move-(8*pawn_dir)) as usize] = Piece{ piece_type: PieceType::None, piece_colour: PieceColour::None}; // remove piece that is taken by en passant
	}

	if start_piece.piece_type == PieceType::Pawn && piece_move.start/8 == pawn_start && piece_move.start+(16*pawn_dir) == piece_move.end{ // make this pos next en passant move if its a double pawn push
		new_en_passant = piece_move.end - (8*pawn_dir);
	}

	// -- promotion stuff --

	match piece_move.flag{
		MoveFlag::RookPromo => board[piece_move.end as usize] = Piece{ piece_type: PieceType::Rook, piece_colour: start_piece.piece_colour }, // change pawn to rook
		MoveFlag::KnightPromo => board[piece_move.end as usize] = Piece{ piece_type: PieceType::Knight, piece_colour: start_piece.piece_colour }, // change pawn to knight
		MoveFlag::BishopPromo => board[piece_move.end as usize] = Piece{ piece_type: PieceType::Bishop, piece_colour: start_piece.piece_colour }, // change pawn to bishop
		MoveFlag::QueenPromo => board[piece_move.end as usize] = Piece{ piece_type: PieceType::Queen, piece_colour: start_piece.piece_colour }, // change pawn to queen
		_ => ()
	}

	// -- king index changes --

	if start_piece.piece_type == PieceType::King{
		match start_piece.piece_colour{
			PieceColour::White => king_indexs.0 = piece_move.end,
			PieceColour::Black => king_indexs.1 = piece_move.end,
			PieceColour::None => ()
		}
	}

	// -- castling stuff --

	// remove castling rights if rook/king is moving
	match start_piece.piece_type{
		PieceType::King => {
			// remove all castling rights for that colour
			match start_piece.piece_colour{
				PieceColour::White => {
					castling_rights.0 = false;
					castling_rights.1 = false;
				},
				PieceColour::Black => {
					castling_rights.2 = false;
					castling_rights.3 = false;
				},
				_ => ()
			}
		},
		PieceType::Rook => {
			// remove castling rights for that side
			match piece_move.start%8{
				0 => { // queen side
					match start_piece.piece_colour{
						PieceColour::White => {
							if piece_move.start/8 == 7{
								castling_rights.1 = false
							}
						},
						PieceColour::Black => {
							if piece_move.start/8 == 0{
								castling_rights.3 = false
							}
						},
						_ => ()
					}
				},
				7 => { // king side
					match start_piece.piece_colour{
						PieceColour::White => {
							if piece_move.start/8 == 7{
								castling_rights.0 = false
							}
						},
						PieceColour::Black => {
							if piece_move.start/8 == 0{
								castling_rights.2 = false
							}
						},
						_ => ()
					}
				},
				_ => ()
			}
		},
		_ => ()
	}
	// remove castling rights if rook is captured
	if capture_piece.piece_type == PieceType::Rook{

		match piece_move.end%8{
			0 => { // queen side
				match capture_piece.piece_colour{
					PieceColour::White => {
						if piece_move.end/8 == 7{
							castling_rights.1 = false
						}
					}, 
					PieceColour::Black => {
						if piece_move.end/8 == 0{
							castling_rights.3 = false
						}
					},
					_ => ()
				}
			},
			7 => { // king side
				match capture_piece.piece_colour{
					PieceColour::White => {
						if piece_move.end/8 == 7{
							castling_rights.0 = false
						}
					},
					PieceColour::Black => {
						if piece_move.end/8 == 0{
							castling_rights.2 = false
						}
					},
					_ => ()
				}
			},
			_ => ()
		}
	}

	// moving rook if move is castling
	if piece_move.flag == MoveFlag::Castling{
		let side =  if piece_move.start - piece_move.end > 0 { -1 } else { 1 }; // which side the king is castling on

		match side{
			-1 => { // queen side
				let rook_index = (piece_move.start/8)*8;

				board[(piece_move.end + 1) as usize] = board[rook_index as usize];
				board[rook_index as usize] = Piece{ piece_type: PieceType::None, piece_colour: PieceColour::None }; 
			},
			1 => { // king side
				let rook_index = ((piece_move.start/8)*8)+7;

				board[(piece_move.end - 1) as usize] = board[rook_index as usize];
				board[rook_index as usize] = Piece{ piece_type: PieceType::None, piece_colour: PieceColour::None }; 
			},
			_ => ()
		}
	}


	(board.to_vec(), new_en_passant, king_indexs, castling_rights)
}

// ------- CHECK AND CHECKMATE -------

fn check_for_checkmate(board: &mut Vec<Piece>, en_passant_move: i32, king_indexs: (i32, i32), colours_turn: PieceColour, castling_rights: (bool,bool,bool,bool)) -> bool{ // returns true if in checkmate 
	let mut legal_moves: Vec<Move> = Vec::new(); // all legal moves that the player can take


	for index in 0i32..64i32{
		if board[index as usize].piece_colour == colours_turn{ // piece is players
			legal_moves.append(&mut legal_move_gen(board, index, en_passant_move, king_indexs, castling_rights))
		}
	}

	if legal_moves.len() == 0{ // return true if no legal moves that can be made
		return true
	}
	else{
		return false
	}
}

// ------- INPUT -------

fn promotion_type_input(message: &str, can_quit: bool) -> MoveFlag{ // returns move flag for which type of promotion selected 
	loop{
		let mut input = String::new();
		println!("\n{}: ", message); // print message that goes with input 
		io::stdin().read_line(&mut input).expect("\x1b[41m--FAILED TO READ INPUT LINE--\x1b[0m"); // get input line from console

		if can_quit && input == "quit\n"{ // go back to piece selection
			return MoveFlag::None;
		}

		match input.trim().to_lowercase().as_str(){
			"rook" => return MoveFlag::RookPromo,
			"knight" => return MoveFlag::KnightPromo,
			"bishop" => return MoveFlag::BishopPromo,
			"queen" => return MoveFlag::QueenPromo,
			_ => println!("-- Pawn cannot promote to {}! --", input.trim())
		}
	}
}

fn algebraic_notation_input(message: &str, can_quit: bool) -> i32{ // returns input from algebraic notations
	let error_message = "\x1b[41m--FAILED TO READ INPUT LINE--\x1b[0m";

	let mut index = 0;
	loop{
		let mut input = String::new();
		println!("\n{}: ", message); // Print message that goes with input
		io::stdin().read_line(&mut input).expect(error_message); // get input line from console

		if can_quit && input == "quit\n"{ // go back to piece selection
			return -1;
		}

		if input.chars().nth(0).expect(error_message).is_alphabetic() && input.chars().nth(1).expect(error_message).is_numeric() && input.trim().len() == 2{ // Making sure input is in the form letter-number
			let character_part = input.chars().nth(0).expect(error_message); // X coordinate
			let number_part = input.chars().nth(1).expect(error_message).to_digit(10).unwrap(); // Y coordinate

			match character_part.to_lowercase().to_string().as_str(){ // matching letter to number
				"a" => index += 0,
				"b" => index += 1,
				"c" => index += 2,
				"d" => index += 3,
				"e" => index += 4,
				"f" => index += 5,
				"g" => index += 6,
				"h" => index += 7,
				_ =>  { println!("-- Not in Algebraic Notation! --"); continue; } // not between a-h
			}

			index += (8-number_part as i32)*8; // calculate index

			break;
		}
		else{
			println!("-- Not in Algebraic Notation! --"); // not in form letter-number
		}
	}
	index
}


fn main() {
	let (mut board, mut colours_turn, mut castling_rights, mut en_passant_move) = decode_fen("rnbqkbnr/p2ppppp/1P6/8/8/3p4/1PP1PPPP/RNBQKBNR w KQkq - "); // rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -

	let piece_art = load_board_art("res/Piece_Art.txt"); // load art from file

	let mut king_indexs: (i32, i32) = (-1, -1); // indexs of the kings
	for index in 0i32..64i32{ // initialising the position of the kings
		let piece = board[index as usize];

		if piece.piece_type == PieceType::King{
			match piece.piece_colour{
				PieceColour::White => king_indexs.0 = index,
				PieceColour::Black => king_indexs.1 = index,
				PieceColour::None => ()
			}
		}
	}


	loop{
		encode_into_fen(&board, colours_turn, castling_rights, en_passant_move);

		if check_for_checkmate(&mut board, en_passant_move, king_indexs, colours_turn, castling_rights){
			print_board(&board, &Vec::new(), &piece_art);	
			match colours_turn{
				PieceColour::White => {
					println!("\x1b[42;30m-- BLACK HAS WON --\x1b[0m");
					break;
				},
				PieceColour::Black => {
					println!("\x1b[42;30m-- WHITE HAS WON --\x1b[0m");
					break;
				},
				_ => ()
			}
		}

		(board, en_passant_move, colours_turn, king_indexs, castling_rights) = selection_iteration(board, colours_turn, en_passant_move, &piece_art, king_indexs, castling_rights);
	}
}