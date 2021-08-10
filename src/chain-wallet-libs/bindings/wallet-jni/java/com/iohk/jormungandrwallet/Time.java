package com.iohk.jormungandrwallet;

public class Time {
    public static class BlockDate {
        public long epoch;
        public long slot;

        public BlockDate(long epoch, long slot) {
            this.epoch = epoch;
            this.slot = slot;
        }
    }

    public native static BlockDate ttlFromDate(long settings, long unixEpoch);
}
