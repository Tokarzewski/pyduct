using System.Collections.Generic;
using System.Globalization;
using System.Text;

namespace Wenta
{
    public class ElectricalData
    {
        public string ComponentId;

        public string DeviceType;

        public double PowerW;

        public double? VoltageV;

        public double? CurrentA;

        public double? PowerFactor;

        public double? FrequencyHz;

        public ElectricalData(string componentId, string deviceType, double powerW)
        {
            if (double.IsNaN(powerW) || double.IsInfinity(powerW) || powerW < 0.0)
                throw new WentaException("power_w must be a non-negative finite value");
            ComponentId = componentId;
            DeviceType = deviceType;
            PowerW = powerW;
        }

        public double? Current()
        {
            double? amps = ComputedCurrent();
            CurrentA = amps;
            return amps;
        }

        public double PowerKw()
        {
            return PowerW / 1000.0;
        }

        internal double? ComputedCurrent()
        {
            if (VoltageV.HasValue && PowerFactor.HasValue
                && VoltageV.Value > 0.0 && PowerFactor.Value > 0.0)
                return PowerW / (VoltageV.Value * PowerFactor.Value);
            return null;
        }
    }

    public class ElectricalSchedule
    {
        private readonly List<ElectricalData> _entries = new List<ElectricalData>();

        public void Add(ElectricalData entry)
        {
            _entries.Add(entry);
        }

        public double TotalPowerW()
        {
            double total = 0.0;
            foreach (ElectricalData e in _entries) total += e.PowerW;
            return total;
        }

        public double? TotalCurrentA()
        {
            double total = 0.0;
            foreach (ElectricalData e in _entries)
            {
                double? c = e.ComputedCurrent();
                if (!c.HasValue) return null;
                total += c.Value;
            }
            return total;
        }

        public int Len()
        {
            return _entries.Count;
        }

        public bool IsEmpty()
        {
            return _entries.Count == 0;
        }

        public IEnumerable<ElectricalData> Iter()
        {
            return _entries;
        }
    }

    public static class Electrical
    {
        public static string AsCsv(ElectricalSchedule schedule)
        {
            var sb = new StringBuilder();
            sb.Append("component_id,device_type,power_w,power_kw,voltage_v,current_a,power_factor,frequency_hz");
            foreach (ElectricalData e in schedule.Iter())
            {
                double? current = e.CurrentA ?? e.ComputedCurrent();

                sb.Append('\n');
                sb.Append(e.ComponentId).Append(',')
                  .Append(e.DeviceType).Append(',')
                  .Append(FmtNum(e.PowerW)).Append(',')
                  .Append(FmtNum(e.PowerKw())).Append(',')
                  .Append(Results.FmtOpt(e.VoltageV)).Append(',')
                  .Append(Results.FmtOpt(current)).Append(',')
                  .Append(Results.FmtOpt(e.PowerFactor)).Append(',')
                  .Append(Results.FmtOpt(e.FrequencyHz));
            }
            return sb.ToString();
        }

        private static string FmtNum(double v)
        {
            return v.ToString(CultureInfo.InvariantCulture);
        }
    }
}
