//! The code of the functions the prelude marks
//! `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော`. Their declarations in the prelude have
//! no block; the compiler gives each one its statements here, written in Zaka
//! with the operator or the verb the function stands for, over the parameter
//! names of its head.

use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Diagnostic;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_session::artifact::EmbeddedFile;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::parser::parse_with;

/// The statements of each function the compiler gives, under its name.
const BODIES: &[(&str, &str)] = &[
    ("ပေါင်း", "က + ခ ကို ပြန်ပေးမည်။"),
    ("နုတ်", "က - ခ ကို ပြန်ပေးမည်။"),
    ("မြှောက်", "က * ခ ကို ပြန်ပေးမည်။"),
    ("စား", "က / ခ ကို ပြန်ပေးမည်။"),
    ("ငယ်မငယ်စစ်", "က < ခ ကို ပြန်ပေးမည်။"),
    ("ကြီးမကြီးစစ်", "က > ခ ကို ပြန်ပေးမည်။"),
    ("ညီမညီစစ်", "က == ခ ကို ပြန်ပေးမည်။"),
    ("စာသားအဖြစ်ပြောင်း", "\"{က}\" ကို ပြန်ပေးမည်။"),
    ("ပြသ", "က ကို ပြသမည်။"),
    ("ထည့်", "ကွက်များ ထဲသို့ က ကို ထည့်မည်။"),
    ("ဖျက်", "ကွက်များ ၏ နောက်ဆုံးအကွက် ကို ဖျက်မည်။"),
    ("အစားထိုး", "စာ ၏ အကွက် စ မှ ဆုံး ထိ ကို အသစ် ဖြင့် အစားထိုးမည်။"),
];

/// Give each marked function of a stdlib module its statements.
///
/// # Arguments
///
/// * `program` - The statements of a stdlib module.
/// * `module` - The name of the module.
/// * `source_map` - Where the text of the statements is registered.
/// * `options` - The lexer options.
/// * `next_id` - The first free node number; moved past the numbers taken.
///
/// # Returns
///
/// The diagnostics of the statements, which a sound compiler never has, and
/// the file of each one, in the order of their registration.
pub(crate) fn give_bodies(
    program: &mut Program, module: &str, source_map: &mut SourceMap,
    options: &LexerOptions, next_id: &mut u32,
) -> (Vec<Diagnostic>, Vec<EmbeddedFile>)
{
    let mut diags = Vec::new();
    let mut files = Vec::new();
    for stmt in &mut program.stmts
    {
        let Stmt::Fn(decl) = stmt
        else
        {
            continue;
        };
        let found = BODIES
            .iter()
            .find(|(name, _)| decl.builtin && *name == decl.name.name.as_str());
        let Some((name, text)) = found
        else
        {
            continue;
        };
        let file_name = FileName::Custom(format!("{module}#{name}"));
        let file = source_map.new_source_file(file_name, (*text).to_string());
        files.push(EmbeddedFile::of(&file));
        let lexed = lex(&file, options);
        let parsed = parse_with(
            &file,
            &lexed,
            *next_id,
            &Default::default(),
            &Default::default(),
        );
        *next_id = parsed.next_id;
        diags.extend(lexed.diagnostics);
        diags.extend(parsed.diagnostics);
        decl.block.stmts = parsed.program.stmts;
    }
    (diags, files)
}
