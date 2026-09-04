using System;

namespace Wenta
{
    /// <summary>Unit converters between SI (wenta's native units) and US customary.
    /// Port of `wenta.units` / `wentamojo.units` — constants identical.</summary>
    public static class Units
    {
        private const double CFM_TO_M3S = 0.0004719474432; // ft^3/min -> m^3/s
        private const double INWC_TO_PA = 249.0889;       // inch H2O (4 °C) -> Pa
        private const double FT_TO_M = 0.3048;
        private const double IN_TO_M = 0.0254;
        private const double FPM_TO_MS = 0.00508;

        public static double CfmToM3s(double cfm) { return cfm * CFM_TO_M3S; }
        public static double M3sToCfm(double m3s) { return m3s / CFM_TO_M3S; }
        public static double InwcToPa(double inwc) { return inwc * INWC_TO_PA; }
        public static double PaToInwc(double pa) { return pa / INWC_TO_PA; }
        public static double FtToM(double ft) { return ft * FT_TO_M; }
        public static double MToFt(double m) { return m / FT_TO_M; }
        public static double InToM(double inches) { return inches * IN_TO_M; }
        public static double MToIn(double m) { return m / IN_TO_M; }
        public static double FpmToMs(double fpm) { return fpm * FPM_TO_MS; }
        public static double MsToFpm(double ms) { return ms / FPM_TO_MS; }
        public static double FToC(double fahrenheit) { return (fahrenheit - 32.0) * 5.0 / 9.0; }
        public static double CToF(double celsius) { return celsius * 9.0 / 5.0 + 32.0; }

        /// <summary>ACH = flowrate × 3600 / room volume.</summary>
        public static double AirChangesPerHour(double flowrateM3s, double volumeM3)
        {
            if (volumeM3 <= 0.0)
                throw new WentaException("volume_m3 must be positive");
            if (flowrateM3s < 0.0)
                throw new WentaException("flowrate_m3s must be non-negative");
            return flowrateM3s * 3600.0 / volumeM3;
        }
    }

    /// <summary>Exception mirroring Python's ValueError contract.</summary>
    public class WentaException : Exception
    {
        public WentaException(string message) : base(message) { }
    }
}
