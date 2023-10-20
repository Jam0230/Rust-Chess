use std::fs;
use std::fmt;
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

#[derive(Debug, Copy, Clone)]
struct Move{
    start: i32,
    end: i32
}

impl fmt::Display for Move{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "{}-{} -> {}-{}", self.start%8, self.start/8, self.end%8, self.end/8)
    }
}

// ------- BOARD PRINTING -------

fn decode_fen(fen_string: &str) -> Vec<Piece>{
    let ranks = fen_string.split("/");
    let mut board: Vec<Piece> = Vec::new();

    for rank in ranks{
        for file in rank.chars(){
            if file.is_numeric(){ // empty space
                for _ in 0..file.to_digit(10).unwrap(){ // add multiple empty spaces to board
                    board.push(Piece{ piece_type: PieceType::None, piece_colour: PieceColour::None});
                }
            } else{ // piece
                let piece_type;
                let piece_colour;

                match file.to_lowercase().to_string().as_str(){
                    "p" => piece_type = PieceType::Pawn, // pawn (p)
                    "r" => piece_type = PieceType::Rook, // rook (r)
                    "n" => piece_type = PieceType::Knight, // knight (n)
                    "b" => piece_type = PieceType::Bishop, // bishop (b)
                    "q" => piece_type = PieceType::Queen, // queen (q)
                    "k" => piece_type = PieceType::King, // king (k)
                    _ => {println!("\x1b[41m-UNEXPECTED VALUE IN FEN STRING--\x1b[0m"); return Vec::new() } // not piece character in fen string (e.g: d)
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
        return board;
    }
    else{
        println!("\x1b[41m--UNEXPECTED LENGTH FEN STRING--\x1b[0m");
        return Vec::new();
    }
}

fn load_piece_art(file_path: &str) -> Vec<[String;9]>{
    let file_contents = fs::read_to_string(file_path).expect("\x1b[41m--COULD NOT READ ART FILE--\x1b[0m");
    let split_contents = file_contents.split("\n"); // Read then split art file contents
    let mut piece_arts: Vec<[String;9]> = Vec::new();

    let mut n = 0;
    let mut piece_art: [String;9] = ["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()]; // Ignore this :3
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

    piece_arts // Return piece arts
}

fn print_board(board: &Vec<Piece>, piece_moves: &Vec<Move>, piece_art: &Vec<[String;9]>){
    let mut lines: Vec<String> = vec!["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()];

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
                    let line = piece_art[art_index as usize][i].clone().replace("#", " ");

                    lines[i] = format!("{}{}", lines[i], line);
                }
            }else{ // actuall pieces
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
                        false => line = line // Dont change if White Square
                    }

                    match board[index].piece_colour{
                        PieceColour::Black => line = line.replace("■", "\x1b[30m■\x1b[0m"), // Replace white square with black square if black piece
                        _ => line = line // Dont change if White Piece/Empty space
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
        lines = vec!["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()];
    }
}

// ------- PIECE MOVE GENERATION -------

fn dist_to_edge(index: i32) -> [i32; 8] {
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

fn sudo_legal_move_gen(board: &Vec<Piece>, index: i32, en_passant_moves: Vec<i32>) -> Vec<Move>{
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

                    piece_moves.push(Move{ start: index, end: new_index });

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

                piece_moves.push(Move{ start: index, end: new_file+new_rank*8})
            }
        },
        PieceType::Pawn => {
            let mut dir = -1; // direction of travel
            let mut start_rank = 6; // rank on which the pawn starts
            let mut en_passant_rank = 3; // rank on which the pawn can perform en passant
            if piece.piece_colour == PieceColour::Black { dir=1; start_rank=1; en_passant_rank=4; } // black pawn values

            if board[(index+(8*dir)) as usize].piece_colour == PieceColour::None{ // single move forward
                piece_moves.push(Move{ start: index, end: index+(8*dir)});

                if board[(index+(16*dir)) as usize].piece_colour == PieceColour::None && index/8 == start_rank{ // double move forward when on starting rank
                    piece_moves.push(Move{ start: index, end: index+(16*dir)});
                }
            }

            for side in [-1i32, 1]{
                if board[(index+side+(8*dir)) as usize].piece_colour != piece.piece_colour && board[(index+side+(8*dir)) as usize].piece_colour != PieceColour::None{ // taking diagonally on each side
                    piece_moves.push(Move{ start: index, end: index+side+(8*dir)});
                }

                if en_passant_moves.contains(&(index+side)) && board[(index+side) as usize].piece_colour != piece.piece_colour && index/8 == en_passant_rank{ // en passant
                    piece_moves.push(Move{ start: index, end: index+side+(8*dir)})
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
                            piece_moves.push(Move{ start: index, end: new_file+new_rank*8});
                        }
                    }
                }
            }
        },
        _ => return Vec::new()
    }

    piece_moves
}

// ------- INPUT -------

fn board_square_input(message: &str) -> i32{
    let error_message = "\x1b[41m--FAILED TO READ INPUT LINE--\x1b[0m";

    let mut index = 0;
    loop{
        let mut input = String::new();
        println!("\n{}: ", message); // Print message that goes with input
        io::stdin().read_line(&mut input).expect(error_message); // Input line from console


        if input.chars().nth(0).expect(error_message).is_alphabetic() && input.chars().nth(1).expect(error_message).is_numeric() && input.trim().len() == 2{ // Making sure input is in the form letter-number (e.g e8)
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
            println!("-- Not in Algebraic Notation! --"); // not in form letter-number (e.g e8)
        }
    }

    index
}

fn main() {
    let board: Vec<Piece> = decode_fen("r1bk3r/p2pBpNp/n4n2/1p1NP2P/6P1/3P4/P1P1K3/q5b1"); // rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR = default r1bk3r/p2pBpNp/n4n2/1p1NP2P/6P1/3P4/P1P1K3/q5b1 = testing

    let piece_art = load_piece_art("res/Piece_Art.txt"); // load art from file

    print_board(&board, &Vec::new(), &piece_art);

    let selected_piece = board_square_input("Input selected piece");

    let test_moves = sudo_legal_move_gen(&board, selected_piece, vec![39]);

    print_board(&board, &test_moves, &piece_art);
}