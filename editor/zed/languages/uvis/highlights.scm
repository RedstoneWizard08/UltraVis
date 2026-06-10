(measure) @constant
(number) @number
(qstr) @string
(float) @number
(comment) @comment
(ident) @label

(res_flag
	"@" @punctuation.special
    "resolution" @type.builtin
)

(fps_flag
	"@" @punctuation.special
    "fps" @type.builtin
)

(song_flag
	"@" @punctuation.special
    "song" @type.builtin
)

(bpm_flag
	"@" @punctuation.special
    "bpm" @type.builtin
)

(marker "x" @operator)
(time "/" @operator)

(note_time) @constant.builtin

(note
	"*" @punctuation.special
    "/" @operator
)

(block
	":" @punctuation
	id: (ident) @label
	"{" @punctuation.bracket
    "}" @punctuation.bracket
)

(measure_marker "->" @operator)

(measure_marker
	"[" @punctuation.bracket
    "]" @punctuation.bracket
)

(repeat
	"#repeat" @keyword
	"(" @punctuation.bracket
    ")" @punctuation.bracket
    "{" @punctuation.bracket
    "}" @punctuation.bracket
)
