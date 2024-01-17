module veryl_testcase_Module42 (
    input  logic i_clk  ,
    input  logic i_rst_n,
    input  logic i_d    ,
    output logic o_d
);
    localparam int unsigned a = pkg::paramA;
    //localparam b: u32 = pkg::paramA;

    delay u0 (
        .i_clk   (i_clk  ),
        .i_rst_n (i_rst_n),
        .i_d     (i_d    ),
        .o_d     (o_d    )
    );
endmodule
