#!/usr/bin/env python3
import json
import csv
import os

# Hard-coded paths
INPUT_JSONL = "out/serializability_stats.jsonl"
SUMMARY_CSV = "out/jsonl_summarizing_table.csv"
OUTPUT_TEX = "tex/tables/big_table_summary.tex"

# map CSV result → LaTeX symbol
SYMBOLS = {
    "serializable": r"\greencmark",
    "not_serializable": r"\xmark",
    "timeout": "?"
}

# categories and their benchmarks (in order)
CATEGORIES = [
    ("Core expressions", ["a1.ser","a2.ser","a3.ser","a4.ser","a5.ser","a6.ser","a7.ser"]),
    ("State machines", ["b1.json","b2.json","b3.json","b4.json"]),
    ("Fred (mixed arithmetic)", ["c1.ser","c2.ser","c3.ser","c4.ser","c5.ser","c6.ser","c7.ser","c8.ser"]),
    ("Circular increment", ["d1.ser","d2.ser","d3.ser","d4.ser","d5.ser","d6.ser","d7.ser"]),
    ("Concurrency \\& locking loops", ["e1.ser","e2.ser","e3.ser","e4.ser","e5.ser","e6.ser","e7.ser","e8.ser"]),
    ("Non-deterministic \\& randomness", ["f1.ser","f2.ser","f3.ser","f4.ser","f5.ser","f6.ser","f7.ser","f8.ser","f9.ser"]),
    ("Networking \\& system protocols", ["g1.ser","g2.ser","g3.ser","g4.ser","g5.ser","g6.ser","g7.ser"])
]

# static features columns (six entries) for each benchmark
FEATURES_COLS = {
    "a1.ser":   "&  & \\cmark &  &  &       &  ",
    "a2.ser":   "&  &        &  &  & \\cmark &  ",
    "a3.ser":   "&  &        &  &  &       &  ",
    "a4.ser":   "&  &        &  &  & \\cmark & \\cmark",
    "a5.ser":   "&  & \\cmark &  &  & \\cmark & \\cmark",
    "a6.ser":   "&  &        &  &  & \\cmark & \\cmark",
    "a7.ser":   "& \\cmark & \\cmark &  &  & \\cmark &  ",
    "b1.json":  "& \\cmark &        &  &  & \\cmark & \\cmark",
    "b2.json":  "& \\cmark &        &  &  & \\cmark & \\cmark",
    "b3.json":  "& \\cmark &        &  &  & \\cmark & \\cmark",
    "b4.json":  "& \\cmark &        &  &  & \\cmark & \\cmark",
    "c1.ser":   "&  & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "c2.ser":   "&  & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "c3.ser":   "&  & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "c4.ser":   "&  & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "c5.ser":   "&  & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "c6.ser":   "&  & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "c7.ser":   "&  & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "c8.ser":   "&  & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "d1.ser":   "& \\cmark &  & \\cmark &  &  \\cmark &  ",
    "d2.ser":   "& \\cmark & \\cmark & \\cmark &  & \\cmark &  ",
    "d3.ser":   "& \\cmark &        & \\cmark &  &   \\cmark &  ",
    "d4.ser":   "& \\cmark &        & \\cmark &  &   \\cmark &  ",
    "d5.ser":   "& \\cmark & \\cmark & \\cmark &  &  \\cmark &  ",
    "d6.ser":   "& \\cmark & \\cmark & \\cmark &  &     \\cmark &  ",
    "d7.ser":   "& \\cmark &        &  &  & \\cmark &  ",
    "e1.ser":   "&  & \\cmark &  &  & \\cmark &  ",
    "e2.ser":   "& \\cmark & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "e3.ser":   "& \\cmark & \\cmark &  & \\cmark &   \\cmark & \\cmark",
    "e4.ser":   "& \\cmark & \\cmark &  &  \\cmark &   \\cmark & \\cmark",
    "e5.ser":   "& \\cmark & \\cmark &  & \\cmark &  \\cmark & \\cmark",
    "e6.ser":   "& \\cmark & \\cmark & \\cmark &  & \\cmark &  ",
    "e7.ser":   "& \\cmark & \\cmark & \\cmark &  & \\cmark &  ",
    "e8.ser":   "&  & \\cmark &  &  &   \\cmark &  ",
    "f1.ser":   "& \\cmark &    \\cmark    & \\cmark &  & \\cmark &  ",
    "f2.ser":   "& \\cmark &   \\cmark     & \\cmark &  & \\cmark &  ",
    "f3.ser":   "&  &        &  & \\cmark &   \\cmark & \\cmark",
    "f4.ser":   "&  &     \\cmark   &  & \\cmark & \\cmark & \\cmark",
    "f5.ser":   "& \\cmark &        & \\cmark &  &       &  ",
    "f6.ser":   "& \\cmark &        & \\cmark &  & \\cmark &  ",
    "f7.ser":   "& \\cmark &        & \\cmark &  &  \\cmark &  ",
    "f8.ser":   "& \\cmark &        & \\cmark &  &   \\cmark &  ",
    "f9.ser":   "& \\cmark &        & \\cmark &  &  \\cmark &  ",
    "g1.ser":   "& \\cmark & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "g2.ser":   "& \\cmark & \\cmark &  & \\cmark & \\cmark & \\cmark",
    "g3.ser":   "& \\cmark & \\cmark & \\cmark & \\cmark & \\cmark & \\cmark",
    "g4.ser":   "& \\cmark & \\cmark & \\cmark & \\cmark & \\cmark & \\cmark",
    "g5.ser":   "& \\cmark & \\cmark & \\cmark & \\cmark &   \\cmark & \\cmark",
    "g6.ser":   "& \\cmark &        & \\cmark & \\cmark & \\cmark &  ",
    "g7.ser":   "& \\cmark &        & \\cmark & \\cmark &       &  ",
}

def summarize_jsonl_to_csv(input_path, output_path):
    """Read JSONL and write benchmark summary CSV."""
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(input_path) as jf, open(output_path, "w", newline="") as cf:
        writer = csv.writer(cf)
        writer.writerow(["benchmark","result","certificate running time","total running time"])
        for line in jf:
            if not line.strip():
                continue
            rec = json.loads(line)
            bm = os.path.basename(rec.get("example",""))
            res = rec.get("result","")
            cert = rec.get("certificate_creation_time_ms","")
            total = rec.get("total_time_ms","")
            writer.writerow([bm, res, cert, total])
    print(f"Wrote CSV summary to {output_path}")

def load_summary(csv_path):
    """Load CSV into dict bench→(res, cert, total)."""
    summary = {}
    with open(csv_path) as cf:
        reader = csv.DictReader(cf)
        for row in reader:
            res = row["result"]
            cert = row["certificate running time"] if res!="timeout" else ""
            total = row["total running time"]     if res!="timeout" else ""
            summary[row["benchmark"]] = (res, cert, total)
    return summary

def generate_table(summary, out_path):
    """Write the LaTeX table using summary data."""
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w") as tex:
        # header
        tex.write(r"""\begin{table}[H]
	\centering
	\small
	% increase horizontal padding between columns
	\setlength{\tabcolsep}{5pt}
	\renewcommand{\arraystretch}{0.9}
	\begin{tabular*}{\textwidth}{@{\extracolsep{\fill}}%
			p{2cm}   % Category
			p{1.5cm} % Benchmark
			c        % Serializable
			c c c c c c % Features
			r r       % Cert, Total
		}
		\toprule
		\multicolumn{2}{c}{\textbf{Benchmark}}
		& \textbf{Serializable}
		& \multicolumn{6}{c}{\textbf{Features}}
		& \multicolumn{2}{c}{\textbf{Runtime (ms)}} \\
		\cmidrule(lr){1-2} \cmidrule(lr){3-3} \cmidrule(lr){4-9} \cmidrule(lr){10-11}
		&
		&
		& If & While & \texttt{?} & Arith & Yield & Multi-req
		& Cert. & Total \\
		\midrule
""")
        # body
        for cat, benches in CATEGORIES:
            tex.write(f"\t\t\\multirow{{{len(benches)}}}{{=}}{{{cat}}}")
            for i, bm in enumerate(benches):
                res, cert, total = summary.get(bm, ("","", ""))
                sym = SYMBOLS.get(res, "")
                feats = FEATURES_COLS[bm]
                # each row: (first has multirow already), others start with &
                prefix = "" if i==0 else "\t\t"
                tex.write(f"{prefix} & \\texttt{{{bm}}} & {sym} {feats} & {cert} & {total} \\\\\n")
           #  only separate categories, not before bottomrule
            tex.write("\t\t\\midrule\n")
        # footer
        # footer: use \bottomrule (no stray \t) and no extra midrule above
        tex.write(r"""\bottomrule
	\end{tabular*}
	\caption{Overview of benchmarks with combined categories and updated serializability markings.}
	\label{tab:benchmarks-all}
\end{table}
""")
    print(f"Wrote LaTeX table to {out_path}")

def main():
    summarize_jsonl_to_csv(INPUT_JSONL, SUMMARY_CSV)
    summary = load_summary(SUMMARY_CSV)
    generate_table(summary, OUTPUT_TEX)

if __name__ == "__main__":
    main()
