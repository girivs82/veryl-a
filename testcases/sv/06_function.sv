module ModuleA ;
    // function without parameter
    function automatic logic [ParamX-1:0] FuncA(
        input  logic [ParamX-1:0] a,
        output logic [ParamX-1:0] b,
        ref    logic [ParamX-1:0] c
    ) ;
        b = a + 1;
        c = a / 1;
        return a + 2;
    endfunction

    // function with parameter
    module FuncB #(
        parameter  int unsigned ParamX  = 1
    );
        function automatic logic [ParamX-1:0] FuncB(
            input  logic [ParamX-1:0] a,
            output logic [ParamX-1:0] b,
            ref    logic [ParamX-1:0] c
        ) ;
            b = a + 1;
            c = a / 1;
            return a + 2;
        endfunction
    endmodule
endmodule