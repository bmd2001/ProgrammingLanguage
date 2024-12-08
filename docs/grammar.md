# Grammar Rules

$$
\begin{gather}
    \langle\:\text{Program}\:\rangle \to \langle\:\text{StmtList}\:\rangle  \\
    \langle\:\text{StmtList}\:\rangle \to 
    \begin{cases} 
        \langle\:\text{Stmt}\:\rangle \\ 
        \langle\:\text{StmtList}\:\rangle\langle\:\text{Stmt}\:\rangle
    \end{cases}\\
    \langle\:\text{Stmt}\:\rangle \to 
    \begin{cases}
        \langle\:\text{ID}\:\rangle = \langle\:\text{ArithmeticExpr}\:\rangle \\
        \text{exit}( \langle\:\text{ArithmeticExpr}\:\rangle)
    \end{cases} \\
    \langle\:\text{ArithmeticExpr}\:\rangle \to 
    \begin{cases}
        \langle\:\text{BaseExpr}\:\rangle \langle\:\text{Op}\:\rangle\langle\:\text{BaseExpr}\:\rangle \\
        \langle\:\text{BaseExpr}\:\rangle
    \end{cases} \\
    \langle\:\text{BaseExpr}\:\rangle \to 
    \begin{cases}
        \langle\:\text{ID}\:\rangle \\
        \langle\:\text{Num}\:\rangle
    \end{cases} \\
    \langle\:\text{ID}\:\rangle \to \texttt{^[a-zA-Z][a-zA-Z0-9]*\$} \\
    \langle\:\text{Num}\:\rangle \to \texttt{[0-9]*} \\
    \langle\:\text{Op}\:\rangle \to
    \begin{cases}
        \times \\
        \div \\
        + \\
        -
    \end{cases}
\end{gather}
$$