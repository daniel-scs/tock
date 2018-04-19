
    _Always (RAMACERE)
    Disabled ()
    Init (RXSTP, TXIN)
      RXSTP => match client.ctrl_setup() {
                 Ok_IN => CtrlReadIn
                 Ok_OUT => CtrlWriteOut
                 _ => stall(); Init
      TXIN => match client.bulk_in() {
                Ok => !FIFOCON
                Delay => BulkInDelay
                _ => stall();
    BulkInDelay (RXSTP)
    CtrlReadIn (TXIN?, NAKOUT)
      NAKOUT => CtrlReadStatus
      TXIN => match client.ctrl_in() {
                Ok(transfer_complete, ..) =>
                  if transfer_complete { !TXIN } else {}
                Delay =>
                  CtrlInDelay
                _ =>
                  Init
    CtrlReadStatus (RXOUT)
      RXOUT => Init
    CtrlWriteOut (RXOUT, NAKIN),
      RXOUT => match client.ctrl_out() {
                 Ok => !RXOUT
                 Delay => !RXOUT
                 _ => stall(); Init
      NAKIN => CtrlWriteStatus
    CtrlWriteStatus (TXIN),
      TXIN => CtrlWriteStatusWait
    CtrlWriteStatusWait (TXIN),
      TXIN => Init
    CtrlInDelay (NAKOUT)
      {}
