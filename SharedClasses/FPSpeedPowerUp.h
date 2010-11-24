//
//  FPSpeedPowerUp.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 7/17/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPGameProtocols.h"

@interface FPSpeedPowerUp : NSObject <FPGameObject>
{
	float x, y;
	int speedUpCounter;
	BOOL isVisible;
}

@end

