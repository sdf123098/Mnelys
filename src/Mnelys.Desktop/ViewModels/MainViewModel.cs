using System.Runtime.InteropServices;

namespace Mnelys.Desktop.ViewModels;

public sealed class MainViewModel : ViewModelBase
{
    public string ProductName { get; } = "Mnelys";

    public string ChineseName { get; } = "忆涟";

    public string Milestone { get; } = "M0 · 第 2 周发布 PoC";

    public string RuntimeDescription { get; } = RuntimeInformation.FrameworkDescription;

    public string PlatformDescription { get; } = RuntimeInformation.OSDescription;
}
