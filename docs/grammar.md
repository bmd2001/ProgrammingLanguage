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
        \langle\:\text{ID}\:\rangle = \langle\:\text{PrimaryExpr}\:\rangle \\
        \text{exit}( \langle\:\text{PrimaryExpr}\:\rangle)\\
       % \text{var } \langle\:\text{Id}\:\rangle = \langle\:\text{PrimaryExpr}\:\rangle
    \end{cases} \\
    \langle\:\text{PrimaryExpr}\:\rangle \to 
    \begin{cases}
        \langle\:\text{ID}\:\rangle \\
        \langle\:\text{Num}\:\rangle \\
        \langle\:\text{ArithmeticExpr}\:\rangle
    \end{cases} \\
    \langle\:\text{ID}\:\rangle \to \texttt{^[a-zA-Z][a-zA-Z0-9]*\$} \\
    \langle\:\text{Num}\:\rangle \to \texttt{[0-9]*} \\
    \langle\:\text{ArithmeticExpr}\:\rangle \to \langle\:\text{Num}\:\rangle \langle\:\text{Op}\:\rangle\langle\:\text{Num}\:\rangle \\
    \langle\:\text{Op}\:\rangle \to
    \begin{cases}
        + \\
        - \\
        \times
    \end{cases}
\end{gather}
$$