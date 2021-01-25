public class LookupSwitch {
	static int i = 0;

	public static void main(String[] args) {
		switch (i) {
			case -1:
				throw new IllegalStateException("-1");
			case 0:
				return;
			case 1:
				throw new IllegalStateException("1");
			case 4:
				throw new IllegalStateException("4");
			case 94132:
				throw new IllegalStateException("94132");
			default:
				throw new IllegalStateException("default");
		}
	}
}
