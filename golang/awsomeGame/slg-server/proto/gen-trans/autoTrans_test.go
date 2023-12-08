package gen_trans

import "testing"

func TestTrans(t *testing.T) {
	Trans()
}

func TestTransOne(t *testing.T) {
	convertMultilineToSingleline("MsgType.kt")
}
