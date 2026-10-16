using System;
using System.Collections.Generic;
using System.Text;

namespace Wenta
{
    public sealed class RoomBalance
    {
        public readonly double SupplyM3s;
        public readonly double ExhaustM3s;

        public RoomBalance(double supplyM3s, double exhaustM3s)
        {
            if (supplyM3s < 0.0)
                throw new WentaException("supply_m3s must be non-negative");
            if (exhaustM3s < 0.0)
                throw new WentaException("exhaust_m3s must be non-negative");
            SupplyM3s = supplyM3s;
            ExhaustM3s = exhaustM3s;
        }

        public double NetM3s() { return SupplyM3s - ExhaustM3s; }

        public bool IsBalanced(double tolerance)
        {
            return tolerance >= 0.0 && Math.Abs(NetM3s()) <= tolerance;
        }

        public double? ImbalanceFraction()
        {
            double peak = Math.Max(SupplyM3s, ExhaustM3s);
            if (peak <= 0.0) return null;
            return NetM3s() / peak;
        }
    }

    public sealed class RoomBalanceSet
    {
        private readonly List<string> _order = new List<string>();
        private readonly List<RoomBalance> _rooms = new List<RoomBalance>();
        private readonly Dictionary<string, double> _volumes =
            new Dictionary<string, double>();

        public void Add(string name, RoomBalance bal)
        {
            _order.Add(name);
            _rooms.Add(bal);
        }

        public void AddWithVolume(string name, RoomBalance bal, double volumeM3)
        {
            if (volumeM3 > 0.0) _volumes[name] = volumeM3;
            Add(name, bal);
        }

        public double TotalSupplyM3s()
        {
            double t = 0.0;
            foreach (RoomBalance b in _rooms) t += b.SupplyM3s;
            return t;
        }

        public double TotalExhaustM3s()
        {
            double t = 0.0;
            foreach (RoomBalance b in _rooms) t += b.ExhaustM3s;
            return t;
        }

        public double OverallNetM3s() { return TotalSupplyM3s() - TotalExhaustM3s(); }

        public bool IsBalanced(double tolerance)
        {
            return tolerance >= 0.0 && Math.Abs(OverallNetM3s()) <= tolerance;
        }

        public string CsvRender()
        {
            var sb = new StringBuilder();
            sb.AppendLine("name,supply_m3s,exhaust_m3s,net_m3s,ach");
            for (int i = 0; i < _order.Count; i++)
            {
                string name = _order[i];
                RoomBalance bal = _rooms[i];
                string ach = "";
                if (_volumes.ContainsKey(name) && _volumes[name] > 0.0)
                    ach = FmtNum(Units.AirChangesPerHour(bal.SupplyM3s, _volumes[name]));
                sb.Append(name).Append(',')
                  .Append(FmtNum(bal.SupplyM3s)).Append(',')
                  .Append(FmtNum(bal.ExhaustM3s)).Append(',')
                  .Append(FmtNum(bal.NetM3s())).Append(',')
                  .Append(ach);
                sb.AppendLine();
            }
            return sb.ToString();
        }

        internal static string FmtNum(double v)
        {
            return v.ToString("0.######", System.Globalization.CultureInfo.InvariantCulture);
        }
    }
}